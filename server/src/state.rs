use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::{select, task};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use std::collections::hash_map::{Entry,HashMap};
use std::error::Error;
use std::fmt::{format, Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

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
}

pub struct ServerState <A: ToSocketAddrs> {
    peers: HashMap<A, Peer>
}

impl<A> ServerState<A> 
where A: ToSocketAddrs + PartialEq + Eq + Display + Hash + Send + Debug + Copy + Sync, 
{
    pub fn new() -> Self {
        println!("made state");
        ServerState { peers: HashMap::new() }
    }

    pub async fn event_handler(
        mut self,
        event_tx: Arc<UnboundedSender<Event<A>>>,
        mut event_rx: UnboundedReceiver<Event<A>>,
        // + Send is just a workaround for now
    ) -> Result<(), Box<dyn Error + Send>> {
        println!("started server");

        while let Some(event) = event_rx.recv().await {
            dbg!("val received!");
            match event {
                Event::NewPeer { addr, stream } => {
                    let contents = format!("User {} has joined the session", addr);

                    match self.peers.entry(addr) {
                        Entry::Occupied(..) => (),
                        Entry::Vacant(entry) => {

                            let (client_tx, mut client_rx) = mpsc::unbounded_channel(); 
                            // let (shutdown_tx, _shutdown_rx) = mpsc::channel(1);

                            entry.insert(
                                Peer {
                                    message_tx: client_tx,
                                }
                            );
                            dbg!("inserted user");

                            self.server_broadcast(contents).await;

                            if let Ok(()) = self.connection_handler(&mut client_rx, stream, Arc::clone(&event_tx), addr).await {
                                let disconnect_msg = format!("{} has been terminated", addr);
                                dbg!("terminated cunt");

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
        event_tx: Arc<UnboundedSender<Event<A>>>,
        addr: A,
    // ) -> Result<(), Box<dyn Error + Send>> {
        // function doesn't really fail besides the tcp receiver receiving a 0 which just 
        // returns Ok() and exits
        ) -> Result<(), ()> {

        let (tcp_rx, mut tcp_tx) = stream.split();

        let mut tcp_rx = BufReader::new(tcp_rx);
        let mut line = String::new();

        // concurrently read and write
        loop {
            select! {
                // read from client
                bytes_read = tcp_rx.read_line(&mut line) => {
                    if bytes_read.unwrap() == 0 {
                        break;
                    }

                    if let Err(e) = event_tx.send(Event::NewMessage { contents: line.clone(), from: addr}) {
                        dbg!("failed sending packet loss: {}\n", e);
                        continue;
                    }
                    
                    line.clear();
                }

                // send to client
                msg = messages.recv() => match msg {
                    Some(msg) => {
                        match tcp_tx.write_all(msg.as_bytes()).await {
                            Ok(()) => (),
                            Err(_) => ()
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }
        }
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