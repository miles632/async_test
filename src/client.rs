use tokio::net::{ToSocketAddrs, TcpStream};
use std::sync::{Arc, RwLock};

struct User <A: impl ToSocketAddrs>{
    stream: Arc<RwLock<ArcTcpStream>>,
    address: A,
    name: String,   

}