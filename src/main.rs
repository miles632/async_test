use std::error::Error;

use futures::channel::mpsc::UnboundedSender;
use futures::executor::block_on;
use futures::future::Join;
use futures::{Future, SinkExt};
// use futures::SinkExt;
// use std::collections::HashMap;
// use std::env;
// use std::io;
// use std::net::SocketAddr;
// use std::sync::Arc;

use state::Event;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle,spawn};
use tokio::sync::{mpsc, Mutex};

pub mod state;

async fn unwrap_future<F, A>(future: F, _sender: UnboundedSender<Event<A>>) -> JoinHandle<()>
where 
    F: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
    // A: Send + 'static,
{
    tokio::task::spawn(async move {
        if let Err(e) = future.await {
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