use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct State {
    pub node_id: String,
    pub node_ids: Vec<String>,
    pub topology: Option<HashMap<String, Vec<String>>>,
    pub messages_recieved: Vec<u64>,
    message_counter: u64,
}

impl From<(String, Vec<String>)> for State {
    fn from(value: (String, Vec<String>)) -> Self {
        return State {
            node_id: value.0,
            node_ids: value.1,
            ..Default::default()
        };
    }
}

impl State {
    pub fn get_and_increment_message_id(&mut self) -> u64 {
        let current = self.message_counter;
        self.message_counter += 1;
        return current;
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum MessageType {
    #[default]
    Noop,
    #[serde(rename = "init")]
    Init,
    #[serde(rename = "init_ok")]
    InitOk,
    #[serde(rename = "echo")]
    Echo,
    #[serde(rename = "echo_ok")]
    EchoOk,
    #[serde(rename = "generate")]
    Generate,
    #[serde(rename = "generate_ok")]
    GenerateOk,
    #[serde(rename = "broadcast")]
    Broadcast,
    #[serde(rename = "broadcast_ok")]
    BroadcastOk,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "read_ok")]
    ReadOk,
    #[serde(rename = "topology")]
    Topology,
    #[serde(rename = "topology_ok")]
    TopologyOk,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: MessageBody,
}

#[derive(Serialize, Deserialize, Default)]
pub struct MessageBody {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology: Option<HashMap<String, Vec<String>>>,
}
