// use tokio::net::unix::SocketAddr;
use std::net::SocketAddr;

use tokio::{io::{stdin, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}, net::TcpStream, select};

pub const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let sockaddr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let mut stream = TcpStream::connect(sockaddr).await.unwrap();

    let (tcp_rx, mut tcp_tx) = stream.split();
    let mut tcp_rx = BufReader::new(tcp_rx).lines();
    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        select! {
            msg_received = tcp_rx.next_line() => match msg_received {
                Ok(msg) => {
                    if let Some(msg) = msg {
                        println!("{}",msg);
                    } else { break; }
                },
                Err(e) => {
                    dbg!(e);
                },
            },

            msg_to_send = stdin.next_line() => match msg_to_send {
                Ok(msg) => { 
                    if msg == None {
                        continue;
                    }
                    match tcp_tx.write_all(msg.unwrap().as_bytes()).await {
                        
                    }
                }
                Err(_) => {continue;}
            }
        }
    } 

    // recieve msg
    let mut buffer:[u8;1024] = [0; 1024];

    Ok(())
}
