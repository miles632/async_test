use std::error::Error;

use futures::future::Join;
use futures::Future;
use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle,spawn};
use tokio::sync::{mpsc, Mutex};

pub mod state;

async fn unwrap_future<F>(future: F) -> JoinHandle<()>
where F: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
{
    tokio::task::spawn(async move {
        if let Err(e) = future.await {
            // log_error(e)   
        }
    })
}

fn log_error<E>(err: Box<E>) where E: Error 
// where E: Error
{
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1").await?;

    loop {
        let (stream, addr) = listener.accept().await?;
    }

    Ok(())
}