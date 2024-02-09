use std::error::Error;
use std::boxed::Box;

use futures::{Future, SinkExt};

use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle,spawn};

use crate::state::Event;

pub mod state;
pub mod client;

async fn unwrap_future<F,A>(future: F, sender: UnboundedSender<Event<A>>) -> JoinHandle<()>
where 
    F: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static + Send,
    A: Send, 
{
    tokio::task::spawn(async move {
        if let Err(e) = future.await {
            sender.send(Event::ServerErrorLogRequest { err: Box::new(e) }).unwrap();
        }
    })
}

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