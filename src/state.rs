use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::{select, task};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use std::collections::hash_map::{Entry,HashMap};
use std::error::Error;
use std::fmt::{format, Debug, Display};
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::{client, ShutdownSignal};

pub enum Event <A: Send>{
    NewPeer {
        addr: A,
        stream: TcpStream,
    },
    PeerDisconnect {
        addr: A,
    },
    NewPrivateMessage {
        from: A,
        to: Vec<A>,
        content: String,
    },
    NewMessage {
        contents: String,
        from: A,
    },
    ServerErrorLogRequest {
        err: Box<dyn Error + Send>,
    }
}

pub struct Peer {
    pub message_tx:  UnboundedSender<String>,
    pub shutdown_tx: Sender<ShutdownSignal>
}

pub struct ServerState <A: ToSocketAddrs> {
    peers: HashMap<A, Peer>
}

impl<A> ServerState<A> 
where A: ToSocketAddrs + PartialEq + Eq + Display + Hash + Send + Debug + Copy + Sync, 
{
    pub fn new() -> Self {
        ServerState { peers: HashMap::new() }
    }

    pub async fn event_handler(
        mut self,
        event_tx: UnboundedSender<Event<A>>,
        mut event_rx: UnboundedReceiver<Event<A>>,
    ) -> Result<(), Box<dyn Error>> {

        while let Some(event) = event_rx.recv().await {
            match event {

                Event::NewPeer { addr, stream } => {
                    let contents = format!("User {} has joined the session", addr);

                    match self.peers.entry(addr) {
                        Entry::Occupied(..) => (),
                        Entry::Vacant(entry) => {

                            let (client_tx, mut client_rx) = mpsc::unbounded_channel(); 
                            let (shutdown_tx, _shutdown_rx) = mpsc::channel(1);

                            entry.insert(
                                Peer {
                                    message_tx: client_tx,
                                    shutdown_tx: shutdown_tx,
                                }
                            );

                            self.server_broadcast(contents).await;

                            if let Ok(()) = self.connection_handler(&mut client_rx, stream, &event_tx, addr).await {
                                let disconnect_msg = format!("{} has been terminated", addr);

                                self.peers.remove(&addr);
                                self.server_broadcast(disconnect_msg).await;
                            }

                        },

                    }
                }

                Event::NewMessage { contents, from } => {
                    let contents = format!("{}: {}", from, contents);
                    self.server_broadcast(contents).await;
                }

                Event::PeerDisconnect { addr } => {
                    todo!()
                }

                Event::ServerErrorLogRequest { err } => {
                    todo!()
                }
                Event::NewPrivateMessage { from, to, content } => {
                    todo!()
                }
            }
        }
        // drop(self);
        Ok(())
    }

    async fn connection_handler(
        &self, 
        messages: &mut UnboundedReceiver<String>, 
        mut stream: TcpStream,
        event_tx: &UnboundedSender<Event<A>>,
        addr: A,
    ) -> Result<(), Box<dyn Error + Send>> {

        let (tcp_rx, mut tcp_tx) = stream.split();

        let mut tcp_rx = BufReader::new(tcp_rx).lines();

        // concurrently read and write
        loop {
            select! {
                // read from client
                line = tcp_rx.next_line() => match line {
                    Ok(line) => { // TODO: finish this and get rid of the unwraps
                        match line {
                            Some(line) => { 
                                if let Err(e) = event_tx.send(Event::NewMessage { contents: line, from: addr}) {
                                    dbg!("failed sending packet loss: {}\n", e);
                                    continue;
                                }
                            },
                            None => continue,
                        }
                    }
                    Err(e) => {
                        event_tx.send(Event::ServerErrorLogRequest { err: Box::new(e) }).unwrap();
                        break;
                    }
                },

                // send to client
                msg = messages.recv() => match msg {
                    Some(msg) => {
                        if let Err(e) = tcp_tx.write_all(msg.as_bytes()).await {
                            dbg!("failed sending packet loss: {}\n", e);
                            continue
                        }
                    }
                    None => {
                        break
                    }
                }
            }
        }
        // TODO: better error handling 
        Ok(())
    }

    async fn server_broadcast(&self, contents: String) {
        let contents = contents.as_str();
        while let Some((addr, peer)) = self.peers.iter().next() {
            if let Err(e) = peer.message_tx.send(contents.to_string()) {
                dbg!("Err: {}, whilst sending: {} to {}", e, contents, addr);
                continue;
            }
        }
    }
}