use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::net::{ToSocketAddrs, TcpStream};

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

enum Event <A: ToSocketAddrs>{
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
    }
}

struct State <A: ToSocketAddrs>{
    peers: HashMap<A, UnboundedSender<String>>,
}

impl<A: ToSocketAddrs> State<A> {
    async fn event_handler(event_rx: UnboundedReceiver<Event>) -> Result<(), Box<dyn Error>> {
        while let Some(event) = event_rx.recv().await {
            match event {
                Event::NewPeer { addr, stream } => {
                    
                }
                Event::PeerDisconnect { addr } => {

                }
                Event:Event::NewMessage { from, to, content } => {

                }
            }
        }
    }
    async fn server_broadcast(contents: String) {

    }
}