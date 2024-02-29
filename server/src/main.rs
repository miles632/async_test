use std::error::Error;
use std::boxed::Box;
use std::net::SocketAddr;

// use futures::{Future, SinkExt};

use state::ServerState;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle,spawn};
use std::rc::Rc;

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
    let mut server_state: ServerState<SocketAddr> = ServerState::new();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event<SocketAddr>>();

    // tx has to be wrapped in some sort of a reference counted pointer,
    // RefCell or Cell is not needed due to sending not requiring any interior mutation
    // rx can stay as is due to it needing mutation for receiving anyway
    let event_tx = Rc::new(event_tx); 

    server_state.event_handler(Rc::clone(&event_tx), event_rx).await.expect("failed setting up event handler");

    loop {
        let event_tx = Rc::clone(&event_tx);
        if let Ok((stream, addr)) = listener.accept().await {
            if let Err(e) = event_tx.send(Event::NewPeer { 
                addr: addr, 
                stream: stream, 
            }) {
                continue;
            }
        } 
    }

    Ok(())
}