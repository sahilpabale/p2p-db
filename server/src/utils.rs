use std::{collections::HashMap, sync::Arc};

use futures_locks::RwLock;
use tokio::net::TcpStream;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    kv::KVStore,
    structs::{message::Message, node::NodeInfo},
};

pub fn get_mac_address() -> Result<String, mac_address::MacAddressError> {
    let mac = mac_address::get_mac_address()?;

    match mac {
        Some(address) => Ok(address.to_string()),
        None => Err(mac_address::MacAddressError::InternalError),
    }
}

pub async fn handle_tcp_stream(
    mut stream: TcpStream,
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    kv_store: Arc<KVStore>,
) {
    let mut buf = vec![0u8; 1024];
    let len = stream.read(&mut buf).await.unwrap();

    let received_msg: Message = serde_json::from_slice(&buf[..len]).unwrap();

    match received_msg {
        Message::HeartbeatResponse => {
            println!("Received Heartbeat");
            let response = Message::HeartbeatResponse;
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream
                .write_all(serialized_response.as_bytes())
                .await
                .unwrap();
        }
        Message::SetValue { key, value } => {
            println!("Received SetValue");
            kv_store.set(key.clone(), value.clone()).await;

            // broadcast sync to all nodes
            let nodes_guard = nodes.read().await;
            for (_, node_info) in nodes_guard.iter() {
                let mut stream = match TcpStream::connect(node_info.tcp_addr).await {
                    Ok(stream) => stream,
                    Err(_) => continue,
                };
                let sync_msg = Message::Sync {
                    key: key.clone(),
                    value: value.clone(),
                };
                let serialized_msg = serde_json::to_string(&sync_msg).unwrap();
                let _ = stream.write_all(serialized_msg.as_bytes()).await;
            }

            let response = Message::ValueResponse {
                value: Some("value set successfully.".to_string()),
            };
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream
                .write_all(serialized_response.as_bytes())
                .await
                .unwrap();
        }
        Message::GetValue { key } => {
            println!("Received GetValue");
            let value = kv_store.get(&key).await;
            let response = Message::ValueResponse { value };
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream
                .write_all(serialized_response.as_bytes())
                .await
                .unwrap();
        }
        Message::Sync { key, value } => {
            println!("Received Sync");
            kv_store.set(key, value).await;
        }
        _ => {}
    }
}
