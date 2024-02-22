use std::error::Error;
use std::boxed::Box;
use std::net::SocketAddr;

// use futures::{Future, SinkExt};

use state::ServerState;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle,spawn};

use crate::state::Event;

pub mod state;
pub mod client;

pub const ADDR: &str = "127.0.0.1";
pub const PORT: &str = "8080";

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

// async fn unwrap_future<F,A>(future: F, sender: Sender<ShutdownSignal>) -> JoinHandle<()>
// where 
//     F: Future<Output = Result<(), Box<dyn Error>>> + 'static ,
// {
//     tokio::task::spawn(async move {
//         if let Err(e) = future.await {
//             sender.send(ShutdownSignal::Cease);
//         }
//     })
// }

pub enum ShutdownSignal{
    Cease
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1").await?;
    // using socket addr as it supports both ipv4 and 6 
    let mut server_state: ServerState<SocketAddr> = ServerState::new();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event<SocketAddr>>();

    server_state.event_handler(event_tx, event_rx).await;

    loop {
        // TODO: wrap event_tx in a Rc of some kind
        let event_tx = event_tx.clone();
        if let Ok((stream, addr)) = listener.accept().await {
            event_tx.send(Event::NewPeer { 
                addr: addr, 
                stream: stream, 
            });
        }
    }

    Ok(())
}