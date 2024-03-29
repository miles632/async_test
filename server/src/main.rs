use std::error::Error;
use std::boxed::Box;
use std::net::SocketAddr;

// use futures::{Future, SinkExt};

// use futures::executor::block_on;
use state::ServerState;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle,spawn};
// use std::rc::Rc;
use std::sync::Arc;

use crate::state::Event;

pub mod state;
pub mod client;

// pub const ADDR: &str = "127.0.0.1";
// pub const PORT: &str = "8080";

// async fn unwrap_future<F,A>(future: F, sender: UnboundedSender<Event<A>>) -> JoinHandle<()>
// where 
//     F: Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'static + Send,
//     A: Send + 'static, 
// {
//     tokio::task::spawn(async move {
//         if let Err(e) = future.await {
//             sender.send(Event::ServerErrorLogRequest { err: e }).unwrap();
//         }
//     })
// }

pub enum ShutdownSignal{
    Cease
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sockaddr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(sockaddr).await?;

    // using socket addr as it supports both ipv4 and 6 
    let mut server_state: ServerState = ServerState::new();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();

    // tx has to be wrapped in some sort of a reference counted pointer,
    // RefCell or Cell is not needed due to sending not requiring any interior mutation
    // rx can stay as is due to it needing mutation for receiving anyway
    // let event_tx = Arc::new(event_tx); 
    // let mut event_rx: &mut UnboundedReceiver<Event<SocketAddr>> = &mut event_rx;

    let event_tx = Arc::new(event_tx);

    tokio::task::spawn({
        server_state.event_handler(Arc::clone(&event_tx), event_rx)
    }
    );
    // server_state.event_handler(Rc::clone(&event_tx), &mut event_rx).await.expect("failed setting up event handler");

    while let Ok((stream, addr)) = listener.accept().await {
        let event_tx = Arc::clone(&event_tx);
        match event_tx.send(Event::NewPeer { 
            addr: addr, 
            stream: stream, 
        }) {
            Ok(()) => {
            },
            Err(e) => {
                dbg!("failed appending peer");
                continue;
            },
        }
    }
    Ok(())
}