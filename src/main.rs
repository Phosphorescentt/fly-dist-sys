mod maelstrom;

use maelstrom::{Message, MessageBody, MessageType, State};
use std::{
    io::{self, BufRead, Write},
    sync::{Arc, Mutex},
};

type StateArc = Arc<Mutex<Option<State>>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialise reading in from stdin!
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdin.lock();

    let stdout = Arc::new(stdout);
    let state = Arc::new(Mutex::new(None));

    loop {
        let mut buf = Vec::new();
        let _res = handle.read_until('\n' as u8, &mut buf);

        let state = state.clone();
        let stdout = stdout.clone();

        let _result = tokio::spawn(async move {
            let response: Message = process(buf, state).await.unwrap();
            let mut json = serde_json::to_string(&response).unwrap();
            // Append newline for Maelstrom protocl
            json.push('\n');
            let chars = json.as_bytes();

            // Write to stdout
            let mut handle = stdout.lock();
            handle.write_all(chars)
        });
    }
    return Ok(());
}

async fn process(buf: Vec<u8>, state: StateArc) -> std::io::Result<Message> {
    let s = std::str::from_utf8(&buf).unwrap();
    let message: Message = serde_json::from_str(&s).unwrap();

    let body: MessageBody = match message.body.message_type {
        MessageType::Init => process_init(&message.body, state),
        MessageType::Echo => process_echo(&message.body, state),
        MessageType::Generate => process_generate(&message.body, state),
        _ => unimplemented!(),
    };

    return Ok(Message {
        // We can just swap them here but in theory we should use
        // state.node_id?
        src: message.dst,
        dst: message.src,
        body,
    });
}

fn process_init(body: &MessageBody, state: StateArc) -> MessageBody {
    // NB: This will reset the state every time we recieve an `init` message.
    let mut state = state.lock().unwrap();
    let _ = state.insert(State::from((
        body.node_id.clone().unwrap(),
        body.node_ids.clone().unwrap(),
    )));

    let msg_id = state.as_mut().unwrap().get_and_increment_message_id();
    return MessageBody {
        message_type: MessageType::InitOk,
        msg_id: Some(msg_id),
        in_reply_to: body.msg_id,
        ..Default::default()
    };
}

fn process_echo(body: &MessageBody, state: StateArc) -> MessageBody {
    let mut state = state.lock().unwrap();
    let msg_id = state.as_mut().unwrap().get_and_increment_message_id();
    return MessageBody {
        message_type: MessageType::EchoOk,
        msg_id: Some(msg_id),
        in_reply_to: body.msg_id,
        echo: body.echo.clone(),
        ..Default::default()
    };
}

fn process_generate(body: &MessageBody, state: StateArc) -> MessageBody {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut state = state.lock().unwrap();
    let msg_id = state.as_mut().unwrap().get_and_increment_message_id();

    let node_id = state.as_ref().unwrap().node_id.clone();
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .to_string();

    let id_string = node_id + "-" + time.as_str();

    return MessageBody {
        message_type: MessageType::GenerateOk,
        msg_id: Some(msg_id),
        in_reply_to: body.msg_id,
        id: Some(id_string),
        ..Default::default()
    };
}
