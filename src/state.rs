use futures::channel::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::net::{ToSocketAddrs, TcpStream};

use std::collections::btree_map::{Entry, OccupiedEntry};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

enum Event <A: ToSocketAddrs, E: Error>{
    NewPeer {
        addr: A,
        stream: Arc<RwLock<TcpStream>>,
    },
    PeerDisconnect {
        addr: A,
    },
    NewMessage {
        from: A,
        to: A,
        content: String,
    },
    ServerErrorLogRequest {
        err: E,
    }
}


struct ServerState <A: ToSocketAddrs>{
    peers:  HashMap<A, UnboundedSender<String>>,
}

impl<A: ToSocketAddrs + PartialEq + Eq + Display + Hash> ServerState<A> {
    async fn event_handler(
        mut self,
        mut event_rx: UnboundedReceiver<Event<A, E>>,
    ) -> Result<(), Box<dyn Error>> 
    {
        while let Some(event) = event_rx.recv().await {
            match event {
                Event::NewPeer { addr, stream } => {
                    let contents = format!("User {} has joined the session", addr);
                    self.server_broadcast(contents).await;

                    
                    match self.peers.entry(addr) {
                        Entry::Occupied(..) => (),
                        Entry::Vacant(entry) => {
                            let (clientrx, clienttx) 
                                = mpsc::unbounded::<String>();
                            entry.insert(clienttx);
                        },
                    }
                }
                Event::PeerDisconnect { addr } => {

                }
                Event::NewMessage { from, to, content } => {

                }
                Event::ServerErrorLogRequest { err } => {
                    self.server_broadcast(err);
                    dbg!(err);
                }
            }
        }
        Ok(())
    }
    async fn server_broadcast<T>(&self, contents: T) {
        while let Some((_addr, sender)) = self.peers.iter().next() {
            sender.send(contents);
        }
    }
}