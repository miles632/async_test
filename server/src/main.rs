use tokio::net::{TcpStream, TcpListener};
use tokio::sync::mpsc;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock};
use tokio::sync::mpsc::{UnboundedSender,UnboundedReceiver};
use tokio::select;
use tokio::task;

use std::error::Error;
use std::boxed::Box;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::ErrorKind::WouldBlock;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;


const LOCAL_ADDR: &str = "127.0.0.1:8080";
const MAX_MSG_SIZE: usize = 4096;

type Sender<T> = UnboundedSender<T>;
type Receiver<T> = UnboundedReceiver<T>;

// type UserTable = HashMap<SocketAddr, Client>;
fn send_all(
    msg:      Vec<u8>,
    mut write_guard: RwLockWriteGuard<HashMap<SocketAddr, Client>>,
) {
    for (_k, v) in write_guard.iter_mut() {
        v.stream.write(&msg);
    }
}

struct Client {
    stream: TcpStream,
    addr: SocketAddr,

    // msg_rcv_buf: Vec<u8>,
    // msg_snd_buf: Vec<u8>,

    connected: bool,
}

impl Client {
    fn new(stream: TcpStream, addr: SocketAddr) -> Client {
        let mut client = Client {
            stream: stream, 
            addr: addr,

            // msg_rcv_buf: vec![],
            // msg_snd_buf: vec![],

            connected: true,
        }; 
        client
    }

    async fn handle(
        &self,
        // msg_recv: Receiver<Vec<u8>>,
        mut user_table: Arc<RwLock<HashMap<SocketAddr, Client>>>,
        ) -> Result<(), Box<dyn Error>> {
        dbg!("handle");
        while let msg_to_broadcast = self.read_from_stream().await {
            println!("sending {}", String::from_utf8_lossy(&msg_to_broadcast));
            let mut write_guard = user_table.write().await;
            send_all(msg_to_broadcast, write_guard);
        }
        Ok(())
    }

    async fn read_from_stream(&self) -> Vec<u8> {
        let mut buf = [0; MAX_MSG_SIZE];
        loop {
            self.stream.readable().await.expect("FUCK");
            dbg!("trying to fucking read");
            match self.stream.try_read(&mut buf) {
                Ok(0) => {
                    dbg!("read 0 bytes");
                    // self.connected = false;
                    //drop(self);
                },

                Ok(bytes) => {
                    println!("read {} bytes", bytes);
                    return buf.to_vec();
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
        mut client_table: RwLockWriteGuard<'_, HashMap<SocketAddr,Client>>) {
        client_table.remove(&self.addr);
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
            // c.handle(Arc::clone(&client_table)).await;
            match client_table.read().await.get(&addr) {
                Some(client) => {
                    if client.handle(Arc::clone(&client_table)).await.is_ok() {
                        client.remove_client_from_map(Arc::clone(&client_table).write().await).await;
                    }
                }

                None => {
                    ()
                }
            }


        });

        // let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    }


    Ok(())
}
