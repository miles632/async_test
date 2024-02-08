use tokio::net::{ToSocketAddrs, TcpStream};
use tokio::sync::mpsc::UnboundedSender;
use std::sync::{Arc, RwLock};

use crate::{Event, ShutdownSignal};
struct User <A: Send> {
    stream: Arc<RwLock<TcpStream>>,
    message_tx: UnboundedSender<Event<A>>,
    shutdown_tx: UnboundedSender<ShutdownSignal>,
}
