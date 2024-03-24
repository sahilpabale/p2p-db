use std::{net::SocketAddr, time::Instant};

pub struct NodeInfo {
    pub last_seen: Instant,
    pub tcp_addr: SocketAddr,
}
