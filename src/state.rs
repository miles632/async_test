use futures::channel::mpsc::Receiver;
use futures::{stream, StreamExt};
use tokio::io::AsyncWriteExt;
// use futures::channel::mpsc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::net::{ToSocketAddrs, TcpStream};

use std::borrow::Borrow;
use std::collections::hash_map::{Entry,HashMap};
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::ShutdownSignal;

pub enum Event <A>{
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
        err: Box<dyn Error>,
    }
}


pub struct ServerState <A: ToSocketAddrs>{
    peers:  HashMap<A, UnboundedSender<String>>,
}

impl<A> ServerState<A> 
where A: ToSocketAddrs + PartialEq + Eq + Display + Hash, {
    async fn event_handler(
        mut self,
        mut event_rx: UnboundedReceiver<Event<A>>,
    ) -> Result<(), Box<dyn Error>> {
        while let Some(event) = event_rx.recv().await {
            match event {
                Event::NewPeer { addr, stream } => {
                    let contents = format!("User {} has joined the session", addr);
                    self.server_broadcast(contents).await;
                    
                    match self.peers.entry(addr) {
                        Entry::Occupied(..) => (),
                        Entry::Vacant(entry) => {
                            let (client_rx, client_tx) = tokio::sync::mpsc::unbounded_channel();
                            entry.insert(client_rx);
                            // self.client_handler(messages, stream)
                        },

                    }
                }
                Event::PeerDisconnect { addr } => {
                    todo!()
                }
                Event::NewMessage { from, to, content } => {
                    todo!()
                }
                Event::ServerErrorLogRequest { err } => {
                    todo!()
                    // self.server_broadcast(err);
                    // dbg!(err);
                }
            }
        }
        drop(self);
        Ok(())
    }

    async fn client_handler(
        &self, 
        messages: &mut Receiver<String>, 
        mut stream: Arc<RwLock<TcpStream>>,
        mut shutdown_rx: Receiver<ShutdownSignal>,
    ) -> Result<(), Box<dyn Error>> {

        // while let Some(msg) = messages.next().await {
        //     if let Ok(mut write_guard) = stream.try_write() {
        //         write_guard.write_all(msg.as_bytes());
        //     } else {
        //         let packetlossmsg = format!("Packet failed to send: {}", msg);
        //         dbg!(packetlossmsg);
        //         continue;
        //     }
        // }
        


        Ok(()) 
    }

    async fn server_broadcast(&self, contents: String) {
        let contents = contents.as_str();
        while let Some((_addr, sender)) = self.peers.iter().next() {
            sender.send(contents.to_string());
        }
    }
}