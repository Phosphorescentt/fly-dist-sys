mod maelstrom;

use maelstrom::{MaelstromState, Message, MessageBody, MessageType, State};
use std::{
    io::{self, BufRead, Stdout, Write},
    sync::{Arc, Mutex},
};

type StateArc = Arc<Mutex<State>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialise reading in from stdin!
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdin.lock();

    let stdout = Arc::new(stdout);
    let state = Arc::new(Mutex::new(State {
        stdout,
        maelstrom_state: None,
    }));

    loop {
        let mut buf = Vec::new();
        let _res = handle.read_until('\n' as u8, &mut buf);

        let state = state.clone();
        let _result = tokio::spawn(async move { process(buf, state).await.unwrap() });
    }
}

fn write_messages(messages: Vec<Message>, stdout: Arc<Stdout>) -> std::io::Result<()> {
    let mut handle = stdout.lock();
    for message in messages.iter() {
        let mut json = serde_json::to_string(&message).unwrap();
        json.push('\n');
        let chars = json.as_bytes();
        let _ = handle.write_all(chars);
    }

    return Ok(());
}

async fn process(buf: Vec<u8>, state: StateArc) -> std::io::Result<()> {
    let s = std::str::from_utf8(&buf).unwrap();
    let message: Message = serde_json::from_str(&s).unwrap();

    let mut state = state.lock().unwrap();
    let messages: Vec<Message> = match message.body.message_type {
        MessageType::Init => process_init(&message, &mut state),
        MessageType::Echo => process_echo(&message, &mut state),
        MessageType::Generate => process_generate(&message, &mut state),
        MessageType::Broadcast => process_broadcast(&message, &mut state),
        MessageType::Read => process_read(&message, &state),
        MessageType::Topology => process_topology(&message, &mut state),
        _ => unimplemented!(),
    };

    return write_messages(messages, state.stdout.clone());
}

fn process_init(message: &Message, state: &mut State) -> Vec<Message> {
    // NB: This will reset the state every time we recieve an `init` message.
    let _ = state.maelstrom_state.insert(MaelstromState::from((
        message.body.node_id.clone().unwrap(),
        message.body.node_ids.clone().unwrap(),
    )));

    let msg_id = state
        .maelstrom_state
        .as_mut()
        .unwrap()
        .get_and_increment_message_id();

    return vec![Message {
        src: message.dst.clone(),
        dst: message.src.clone(),
        body: MessageBody {
            message_type: MessageType::InitOk,
            msg_id: Some(msg_id),
            in_reply_to: message.body.msg_id,
            ..Default::default()
        },
    }];
}

fn process_echo(message: &Message, state: &mut State) -> Vec<Message> {
    let msg_id = state
        .maelstrom_state
        .as_mut()
        .unwrap()
        .get_and_increment_message_id();

    return vec![Message {
        src: message.dst.clone(),
        dst: message.src.clone(),
        body: MessageBody {
            message_type: MessageType::EchoOk,
            msg_id: Some(msg_id),
            in_reply_to: message.body.msg_id,
            echo: message.body.echo.clone(),
            ..Default::default()
        },
    }];
}

fn process_generate(message: &Message, state: &mut State) -> Vec<Message> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let maelstrom_state = state.maelstrom_state.as_mut().unwrap();
    let msg_id = maelstrom_state.get_and_increment_message_id();
    let node_id = maelstrom_state.node_id.clone();
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .to_string();

    let id_string = node_id + "-" + time.as_str();

    return vec![Message {
        src: message.dst.clone(),
        dst: message.src.clone(),
        body: MessageBody {
            message_type: MessageType::GenerateOk,
            msg_id: Some(msg_id),
            in_reply_to: message.body.msg_id,
            id: Some(id_string),
            ..Default::default()
        },
    }];
}

fn process_broadcast(message: &Message, state: &mut State) -> Vec<Message> {
    state
        .maelstrom_state
        .as_mut()
        .unwrap()
        .messages_recieved
        .push(message.body.message.unwrap());

    return vec![Message {
        src: message.dst.clone(),
        dst: message.src.clone(),
        body: MessageBody {
            message_type: MessageType::BroadcastOk,
            in_reply_to: message.body.msg_id,
            ..Default::default()
        },
    }];
}

fn process_read(message: &Message, state: &State) -> Vec<Message> {
    let state = state.maelstrom_state.as_ref().expect("No state");
    let messages = state.messages_recieved.clone();
    return vec![Message {
        src: message.dst.clone(),
        dst: message.src.clone(),
        body: MessageBody {
            message_type: MessageType::ReadOk,
            in_reply_to: message.body.msg_id,
            messages: Some(messages),
            ..Default::default()
        },
    }];
}

fn process_topology(message: &Message, state: &mut State) -> Vec<Message> {
    let _ = state
        .maelstrom_state
        .as_mut()
        .unwrap()
        .topology
        .insert(message.body.topology.clone().unwrap());

    return vec![Message {
        src: message.dst.clone(),
        dst: message.src.clone(),
        body: MessageBody {
            message_type: MessageType::TopologyOk,
            in_reply_to: message.body.msg_id,
            ..Default::default()
        },
    }];
}
