use tokio::net::{TcpStream, TcpListener};
use tokio::sync::{RwLockWriteGuard, RwLock};
use tokio::task;
use tokio::io::AsyncWriteExt;


use std::error::Error;
use std::boxed::Box;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::io::ErrorKind::WouldBlock;
use std::sync::Arc;

const LOCAL_ADDR: &str = "127.0.0.1:8080";
const MAX_MSG_SIZE: usize = 4096;

struct Client {
    stream: TcpStream,
    addr: SocketAddr,
}

impl Client {
    fn new(stream: TcpStream, addr: SocketAddr) -> Client {
        let mut client = Client {
            stream: stream, 
            addr: addr,
        };
        client
    }

    async fn handle(
        &self,
        mut user_table: Arc<RwLock<HashMap<SocketAddr, Client>>>,
        ) -> Result<(), Box<dyn Error>>
    {
        while let Ok(msg_to_broadcast) = self.read_from_stream().await {
            let mut write_guard = user_table.write().await;
            send_all(msg_to_broadcast, write_guard, Some(self.addr));
        }
        Ok(())
    }

    async fn read_from_stream(&self) -> io::Result<Vec<u8>>{
        let mut buf = [0; MAX_MSG_SIZE];
        loop {
            self.stream.readable().await?;
            match self.stream.try_read(&mut buf) {
                Ok(0) => {
                    continue;
                },

                Ok(bytes) => {
                    return Ok(buf.to_vec());
                },
                
                Err(e) => {
                    if e.kind() == WouldBlock {
                        continue;
                    } else {
                        eprintln!("{}", e);
                    }
                },
            }
        }
    }

    async fn remove_client_from_map(
        &self,
        mut client_table: RwLockWriteGuard<'_, HashMap<SocketAddr,Client>>)
    {
        client_table.remove(&self.addr);
    }
}

fn send_all(
    msg:                Vec<u8>,
    mut write_guard:    RwLockWriteGuard<HashMap<SocketAddr, Client>>,
    from:               Option<SocketAddr>,
) {
    // if address is present concatenate with the preexisting msg vec
    if from != None {
        let str = format!("{}: ", from.unwrap());
        let mut str_vec = str.as_bytes().to_vec();
        str_vec.extend(msg.iter());
        let msg = str_vec;
    }

    for (_k, v) in write_guard.iter_mut() {
        v.stream.write(&msg);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let listener = TcpListener::bind(LOCAL_ADDR).await?;

    //let (tx, mut rx) = unbounded_channel::new();
    let mut client_table: Arc<RwLock<HashMap<SocketAddr, Client>>>  =
        Arc::new(RwLock::new(HashMap::new()));

    while let Ok((mut stream, addr)) = listener.accept().await {
        println!("new user: {}", addr);
        let mut c = Client::new(stream, addr);

        let mut client_table = Arc::clone(&client_table);
        client_table.write().await.insert(c.addr, c);

        task::spawn(async move {
            match client_table.read().await.get(&addr) {
                Some(client) => {
                    if client.handle(Arc::clone(&client_table)).await.is_ok() {
                        client.remove_client_from_map(Arc::clone(&client_table).write().await).await;

                        let disconnect_msg = format!("user {} disconnected", client.addr);
                        send_all(
                            disconnect_msg.as_bytes().to_vec(),
                            client_table.write().await,
                            None
                        );
                    }
                }

                None => {
                    ()
                }
            }
        });
    }

    Ok(())
}
