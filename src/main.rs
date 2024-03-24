use futures_locks::RwLock;
use kv::KVStore;
use mac_address::get_mac_address;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration, vec};
use structs::{message::Message, node::NodeInfo};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream, UdpSocket},
};
use utils::handle_tcp_stream;

pub mod kv;
pub mod structs;
pub mod utils;

const BROADCAST_ADDR: &str = "255.255.255.255:8888";
const TCP_PORT: u16 = 9000;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let socket = UdpSocket::bind(&local_addr).await?;
    socket.set_broadcast(true)?;

    // Initialize the key-value store
    let kv_store = Arc::new(KVStore::new());
    let nodes = Arc::new(RwLock::new(HashMap::<String, NodeInfo>::new()));

    // Use Arc to share the socket among tasks.
    let socket = Arc::new(socket);
    let socket_for_broadcast = socket.clone();

    tokio::spawn(async move {
        match get_mac_address() {
            Ok(node_name) => {
                let tcp_addr = format!("{}:{}", "0.0.0.0", TCP_PORT).parse().unwrap();
                let msg = Message::Handshake {
                    node_name: node_name.unwrap().to_string(),
                    tcp_addr,
                };

                let serialized_msg = serde_json::to_string(&msg).unwrap();

                loop {
                    println!("sending udp broadcast...");
                    socket_for_broadcast
                        .send_to(serialized_msg.as_bytes(), BROADCAST_ADDR)
                        .await
                        .unwrap();

                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
            Err(e) => {
                eprintln!("error fetching MAC address: {:?}", e)
            }
        }
    });

    let nodes_clone = nodes.clone();

    tokio::spawn(async move {
        let listener = TcpListener::bind(("0.0.0.0", TCP_PORT)).await.unwrap();
        println!("TCP listener started.");
        while let Ok((stream, _)) = listener.accept().await {
            println!("Accepted new TCP connection.");
            tokio::spawn(handle_tcp_stream(
                stream,
                nodes_clone.clone(),
                kv_store.clone(),
            ));
        }
    });

    let mut buf = vec![0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        println!("Received data on UDP from {}", addr);

        let received_msg: Message = serde_json::from_slice(&buf[..len])?;

        let local_node_name = get_mac_address()?;

        if let Message::Handshake {
            node_name,
            tcp_addr,
        } = received_msg
        {
            // ignore packets from ourselves

            if node_name == local_node_name.unwrap().to_string() {
                continue;
            }

            println!("Received handshake from: {}", node_name);
            {
                let mut nodes_guard = nodes.write().await;
                nodes_guard.insert(
                    node_name.clone(),
                    NodeInfo {
                        last_seen: std::time::Instant::now(),
                        tcp_addr,
                    },
                );
            }

            let greeting = Message::Greeting;
            let serialized_greeting = serde_json::to_string(&greeting).unwrap();
            socket
                .send_to(serialized_greeting.as_bytes(), &addr)
                .await?;

            // Start heartbeat for this node
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    println!("Sending heartbeat to {}", tcp_addr);
                    let mut stream = TcpStream::connect(tcp_addr).await.unwrap();
                    let heartbeat_msg = Message::Heartbeat;
                    let serialized_msg = serde_json::to_string(&heartbeat_msg).unwrap();
                    stream.write_all(serialized_msg.as_bytes()).await.unwrap();
                }
            });
        }
    }
}
