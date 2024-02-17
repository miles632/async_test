use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::select;
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
    NewMessage {
        from: A,
        to: Vec<A>,
        content: String,
    },
    ServerErrorLogRequest {
        err: Box<dyn Error + Send>,
    }
}

pub struct Peer {
    pub message_tx:  UnboundedSender<String>,
    pub shutdown_tx: Sender<ShutdownSignal>
}

pub struct ServerState <A>
where 
    A: ToSocketAddrs + Send
{
    // peers:  HashMap<A, UnboundedSender<String>>,
    peers: HashMap<A, Peer>
    // peers: HashMap<A, (UnboundedSender<String>, UnboundedSender<ShutdownSignal>)>
}

impl<A> ServerState<A> 
where A: ToSocketAddrs + PartialEq + Eq + Display + Hash + Send + Debug + Copy, 
{

    async fn event_handler(
        mut self,
        // mut event_rx: UnboundedReceiver<Event<A>>,
    ) -> Result<(), Box<dyn Error>> {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event<A>>();

        while let Some(event) = event_rx.recv().await {
            match event {

                Event::NewPeer { addr, stream } => {
                    let contents = format!("User {} has joined the session", addr);

                    // let usr_event_tx = &event_tx;
                    
                    match self.peers.entry(addr) {
                        Entry::Occupied(..) => (),
                        Entry::Vacant(entry) => {
                            let (client_tx, mut client_rx) = mpsc::unbounded_channel(); 
                            let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
                            entry.insert(
                                Peer {
                                    message_tx: client_tx,
                                    shutdown_tx: shutdown_tx,
                                }
                            );
                            if let Ok(()) = self.connection_handler(&mut client_rx, stream, &event_tx).await {
                                self.peers.remove(&addr);
                            }
                            self.server_broadcast(contents).await;
                        },

                    }
                }

                Event::NewMessage { from, to, content } => {
                    for addr in to {
                        let str = format!("{}: {}\n", from, content);

                        let peer = self.peers.get(&addr).unwrap();
                        if let Err(_e) = peer.message_tx.send(str) {
                            continue;
                        }
                    }
                }

                Event::PeerDisconnect { addr } => {
                    todo!()
                }

                Event::ServerErrorLogRequest { err } => {
                    todo!()
                }
            }
        }
        // drop(self);
        Ok(())
    }

    // TODO: add server broadcasting alongside an event sender to state
    async fn connection_handler(
        &self, 
        messages: &mut UnboundedReceiver<String>, 
        mut stream: TcpStream,
        event_tx: &UnboundedSender<Event<A>>,
        // mut shutdown_tx: Sender<ShutdownSignal>,
    ) -> Result<(), Box<dyn Error>> {

        let (tcp_rx, mut tcp_tx) = stream.split();

        let mut tcp_rx = BufReader::new(tcp_rx).lines();

        // concurrently read and write
        loop {
            select! {
                // read from client
                line = tcp_rx.next_line() => match line {
                    Ok(line) => { // TODO: finish this and get rid of the unwraps
                        // event_tx.send(Event::NewMessage { from: (), to: (), content: () }).unwrap();
                        todo!()
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
                            dbg!("failed sending packet loss\n");
                            continue
                        }
                    }
                    None => {
                        break
                    }
                }
            }
        }
        // shutdown_tx.send(ShutdownSignal::Cease);
        Ok(())
    }

    async fn server_broadcast(&self, contents: String) {
        let contents = contents.as_str();
        while let Some((_addr, peer)) = self.peers.iter().next() {
            peer.message_tx.send(contents.to_string());
        }
    }
}