use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::sync::mpsc::{UnboundedSender, Sender};
use std::sync::{Arc, RwLock};

use crate::ShutdownSignal;
use crate::state::Event;

// pub struct Peer {
//     // stream: Arc<RwLock<TcpStream>>,
//     pub message_tx:  UnboundedSender<String>,
//     pub shutdown_tx: Sender<ShutdownSignal>
//     // shutdown_tx: UnboundedSender<ShutdownSignal>,
// }


