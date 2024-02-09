use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::select;
use tokio::io::AsyncWriteExt;

use std::collections::hash_map::{Entry,HashMap};
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::{client, ShutdownSignal};
use crate::client::Peer;



pub enum Event <A: Send>{
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
        err: Box<dyn Error + Send>,
    }
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
where A: ToSocketAddrs + PartialEq + Eq + Display + Hash + Send, 
{

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
                            let (client_tx, mut client_rx) 
                                = mpsc::unbounded_channel(); 
                            let (shutdown_tx, shutdown_rx) 
                                = mpsc::channel(1);
                            entry.insert(
                                Peer {
                                    message_tx: client_tx,
                                    shutdown_tx: shutdown_tx,
                                }
                            );
                            self.client_handler(&mut client_rx, stream, shutdown_rx);
                        },

                    }
                }

                Event::NewMessage { from, to, content } => {
                    todo!()
                }

                Event::PeerDisconnect { addr } => {
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

    // TODO: add server broadcasting alongside an event sender to state
    async fn client_handler(
        &self, 
        messages: &mut UnboundedReceiver<String>, 
        mut stream: Arc<RwLock<TcpStream>>,
        mut shutdown_rx: Receiver<ShutdownSignal>,
    ) -> Result<(), Box<dyn Error>> {

        let local_addr = stream.try_read().unwrap().local_addr()?;

        // poll the else branch unless .recv() is ready
        select! {
            _ = shutdown_rx.recv() => {
                drop(stream);
                drop(messages);
                dbg!("client {} has been terminated", local_addr);
            }

            else => {
                loop {
                    while let Some(msg) = messages.recv().await {
                        if let Ok(mut wguard) = stream.try_write() {
                            wguard.write_all(msg.as_bytes());
                        } else {
                            let packetlossmsg = format!("Packet failed to send: {}", msg);
                            dbg!(packetlossmsg);
                            continue;
                        }
                    }
                }
            }

        }
        Ok(()) 
    }

    async fn server_broadcast(&self, contents: String) {
        let contents = contents.as_str();
        while let Some((_addr, peer)) = self.peers.iter().next() {
            peer.message_tx.send(contents.to_string());
        }
    }
}