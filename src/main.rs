use std::error::Error;
use std::boxed::Box;

// use futures::{Future, SinkExt};

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

    loop {
        let (stream, addr) = listener.accept().await?;
    }

    Ok(())
}