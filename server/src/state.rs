#![warn(unused, unused_imports)]
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::{select, task};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncRead};

use std::collections::hash_map::{Entry,HashMap};
use std::error::Error;
use std::fmt::{format, Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

use std::net::SocketAddr;

use crate::{client, ShutdownSignal};

// #[derive(AsyncRead)]
pub enum Event{
    NewPeer {
        addr: SocketAddr,
        stream: TcpStream,
    },
    DropUser {
        addr: SocketAddr,
    },
    NewPrivateMessage {
        from: SocketAddr,
        to: Vec<SocketAddr>,
        content: String,
    },
    AllMessage {
        contents: String,
        from: SocketAddr,
    },
    UserAppend {
        usr_addr: SocketAddr,
        sender: UnboundedSender<String>,
    }
}
    // ServerErrorLogRequest {
    //     err: Box<dyn Error + Send>,
    // },

// #[derive(Clone)]
// pub struct Peer {
//     pub message_tx:  UnboundedSender<String>,
// }

#[derive(Clone)]
pub struct ServerState {
    peers: HashMap<SocketAddr, UnboundedSender<String>>
}

impl ServerState 
// where A: ToSocketAddrs + PartialEq + Eq + Display + Hash + Send + Debug + Sync + AsyncRead 
// + 'static 
{

    pub fn new() -> Self {
        println!("made state");
        ServerState { peers: HashMap::new() }
    }

    pub async fn event_handler(
        &mut self,
        event_tx: Arc<UnboundedSender<Event>>,
        mut event_rx: UnboundedReceiver<Event>,
        // + Send is just a workaround for now
    ) -> Result<(), Box<dyn Error + Send>> {
        println!("started server");

        while let Some(event) = event_rx.recv().await {
            dbg!("val received!");
            match event {
                Event::NewPeer { addr, stream } => {
                    let event_tx = Arc::clone(&event_tx);

                    task::spawn( async move{
                        let (client_tx, mut client_rx) = mpsc::unbounded_channel(); 

                        event_tx.send(Event::UserAppend { usr_addr: addr, sender: client_tx }).unwrap();

                        if let Ok(()) = ServerState::connection_handler(&mut client_rx, stream, event_tx, addr).await {
                            // let disconnect_msg = format!("user {} has disconnected", addr);
                            // self.server_broadcast(disconnect_msg).await;
                            // self.peers.remove(&addr);
                            let event_tx = Arc::clone(&event_tx);
                            event_tx.send(Event::DropUser { addr: addr }).unwrap();
                        }
                    });
                }

                Event::AllMessage { contents, from } => {
                    let contents = format!("{}: {}", from, contents);
                    // self.server_broadcast(contents).await;
                }
                Event::UserAppend { usr_addr, sender } => {
                    if let None = self.peers.insert(usr_addr, sender) {
                        let connect_msg = format!("User {} joined", usr_addr);
                        self.server_broadcast(connect_msg);
                    }
                }
                Event::DropUser { addr } => {todo!()}
                // Event::ServerErrorLogRequest { err } => {
                //     todo!()
                // }
                Event::NewPrivateMessage { from, to, content } => {
                    todo!()
                }
            }
        }
        // drop(self);
        Ok(())
    }

    async fn connection_handler(
        // &mut self, 
        messages: &mut UnboundedReceiver<String>, 
        mut stream: TcpStream,
        event_tx: Arc<UnboundedSender<Event>>,
        addr: SocketAddr,
        // function doesn't really fail besides the tcp receiver receiving a 0 which just 
        // returns Ok() and exits
        ) -> Result<(), ()> {

        let (tcp_rx, mut tcp_tx) = stream.split();

        let mut tcp_rx = BufReader::new(tcp_rx);
        let mut line = String::new();

        let hello_str = "hello".to_string();
        tcp_tx.write_all(hello_str.as_bytes()).await.unwrap();


        // concurrently read and write
        loop {
            select! {
                // read from client
                bytes_read = tcp_rx.read_line(&mut line) => {
                    if bytes_read.unwrap() == 0 {
                        break;
                    }

                    if let Err(e) = event_tx.send(Event::AllMessage { contents: line.clone(), from: addr}) {
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
        while let Some((addr, sender)) = self.peers.iter().next() {
            if let Err(e) = sender.send(contents.to_string()) {
                dbg!("Err: {}, whilst sending: {} to {}", e, contents, addr);
                continue;
            }
        }
    }
}