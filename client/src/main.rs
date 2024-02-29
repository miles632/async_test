// use tokio::net::unix::SocketAddr;
use std::net::SocketAddr;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

pub const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let sockaddr = SocketAddr::from(([127, 0, 0, 1], PORT));

    if let Ok(stream) = TcpStream::connect(sockaddr).await {
        println!("connected to server at port: {}", PORT);
    }
        
    // println!("Connected to server at port {}", PORT);

    //test
    let message = "hello";
    stream.write_all(message.as_bytes()).await?;

    // recieve msg
    let mut buffer:[u8;1024] = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let n = String::from_utf8_lossy(&buffer[..n]);
    println!("Received: {}", n);

    Ok(())
}
