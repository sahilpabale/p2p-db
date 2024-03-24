use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

// defines what types of messages all the nodes in the network can communicate
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    // the initial handshake message to know about the nodes in the network
    Handshake {
        node_name: String,
        tcp_addr: SocketAddr,
    },
    // greet the nodes with their presence
    Greeting,
    // ping other nodes
    Heartbeat,
    // pong from other nodes
    HeartbeatResponse,
    // to set a value in db with a key and its value
    SetValue {
        key: String,
        value: String,
    },
    // to get a value from the db using its key
    GetValue {
        key: String,
    },
    // the optional string response from the get command
    ValueResponse {
        value: Option<String>,
    },
    // synchronize other nodes with the data from a node
    Sync {
        key: String,
        value: String,
    },
}
