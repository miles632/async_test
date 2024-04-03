use std::net::SocketAddr;
// use std::io::Result;
use std::error::Error;

use tokio::{
    io::{stdin, Stdin, AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::TcpStream, select
};

pub const PORT: u16 = 8080;

// ANSI escape code constants
// println!("\x1b[0;31mSO\x1b[0m");
const GREEN: &str = "\x1b[0;32m";
const RESET: &str = "\x1b[0m";


// macro to print the received/inputted tcp messages with color formatting
macro_rules! print_socket_message {
    // if the ADDR is in a seperate string
    ($addr: ident, $text: ident) => {
        // let $addr = format!("{:?}",$addr).green();
        println!(
            "{:?}{:?}{:?}: {:?}",
            GREEN,
            $addr,
            RESET,
            $text
        )
    };

    ($text: ident) => {
        // takes a string usually "ADDR:TEXT" and splits by colon, beware that ADDR also has a colon
        // between the port number and the address hence the joining first 2 elements into a String
        let vec = $text.split(':').collect::<Vec<&str>>();
        let addr = String::from(vec[0].to_owned() + vec[1]).push(':');
        let msg = String::from(vec[2..].join("'"));

        // let addr = format!("{:?}", addr);

        println!(
            "{:?}{:?}{:?}: {:?}",
            GREEN,
            addr,
            RESET,
            msg,
        )
    };
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let sockaddr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let mut stream = TcpStream::connect(sockaddr).await.unwrap();

    let my_addr = &stream.peer_addr().unwrap();


    let (tcp_rx, mut tcp_tx) = stream.split();
    let mut tcp_rx = BufReader::new(tcp_rx).lines();
    let mut stdin = BufReader::new(stdin()).lines();


    loop {
        select! {
            msg_received = tcp_rx.next_line() => match msg_received {
                Ok(msg) => {
                    if let Some(msg) = msg {
                        print_socket_message!(msg);
                    } else { break; }
                },
                Err(e) => {
                    dbg!(e);
                },
            },

            msg_to_send = stdin.next_line() => match msg_to_send {
                Ok(msg) => {
                    if let Some(msg) = msg {
                        print_socket_message!(my_addr, msg);

                        match tcp_tx.write_all(msg.as_bytes()).await {
                            Ok(()) => (),
                            Err(_) => ()
                        }
                    }

                }
                Err(_) => {continue;}
            }
        }
    }

    // recieve msg
    // let mut buffer:[u8;1024] = [0; 1024];

    Ok(())
}

