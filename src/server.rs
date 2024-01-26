use std::sync::{Arc, Mutex};

use crate::{cli::CliParam, database::Database};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use self::inbound_message::{config_message::ConfigMessage, InboundMessage};
use self::outbound_message::OutboundMessage;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: u32 = 6379;
const MB: usize = 1024 * 1024;

mod inbound_message;
mod outbound_message;
mod resp;

pub async fn start_database(cli_params: Vec<CliParam>) -> anyhow::Result<()> {
    let mut database = Database::new();
    database.config_setup(&cli_params);
    let database = Arc::new(Mutex::new(database));

    let listener = TcpListener::bind(format!("{}:{}", DEFAULT_IP, DEFAULT_PORT)).await?;
    println!(
        "-> Started database server at {}:{}",
        DEFAULT_IP, DEFAULT_PORT
    );

    loop {
        let (mut stream, _) = listener.accept().await?;
        let task_database = Arc::clone(&database);
        tokio::spawn(async move { handle_stream(&task_database, &mut stream).await });
    }
}

async fn handle_stream(
    database: &Arc<Mutex<Database>>,
    stream: &mut TcpStream,
) -> anyhow::Result<()> {
    loop {
        let mut buffer: Vec<u8> = Vec::with_capacity(1 * MB);
        let bytes_read = stream.read_buf(&mut buffer).await?;
        let inbound_message = InboundMessage::try_from(&buffer[..bytes_read])?;
        println!("-> Inbound message: {:?}", inbound_message);

        let outbound_message = handle_message(&database, &inbound_message)?;
        println!("-> Outbound message: {:?}", outbound_message);
        let outbound_message_bytes: Vec<u8> = outbound_message.into();
        stream.write_all(&outbound_message_bytes).await?;
    }
}

fn handle_message(
    database: &Arc<Mutex<Database>>,
    message: &InboundMessage,
) -> anyhow::Result<OutboundMessage> {
    match message {
        &InboundMessage::Config(ref config_message) => {
            handle_action_config(database, config_message.clone())
        }
        &InboundMessage::Ping => Ok(OutboundMessage::Pong),
        &InboundMessage::Echo(ref string) => Ok(OutboundMessage::Echo(string.into())),
        &InboundMessage::Set {
            ref key,
            ref value,
            ref expires_at,
        } => handle_action_set(database, key.into(), value.into(), *expires_at),
        &InboundMessage::Get { ref key } => handle_action_get(database, key.into()),
        &InboundMessage::Keys { ref pattern } => handle_action_keys(database, pattern.into()),
    }
}

fn handle_action_config(
    database: &Arc<Mutex<Database>>,
    config_message: ConfigMessage,
) -> anyhow::Result<OutboundMessage> {
    let Ok(database) = database.lock() else {
        anyhow::bail!("Failed to lock database");
    };
    match config_message {
        ConfigMessage::Get { key } => {
            let value = database.config_get(&key);
            Ok(OutboundMessage::ConfigGet { key, value })
        }
    }
}

fn handle_action_set(
    database: &Arc<Mutex<Database>>,
    key: String,
    value: String,
    expires_at: Option<u128>,
) -> anyhow::Result<OutboundMessage> {
    let Ok(mut database) = database.lock() else {
        anyhow::bail!("Failed to lock database");
    };
    database.set(key, value, expires_at)?;
    Ok(OutboundMessage::Ok)
}

fn handle_action_get(
    database: &Arc<Mutex<Database>>,
    key: String,
) -> anyhow::Result<OutboundMessage> {
    let Ok(mut database) = database.lock() else {
        anyhow::bail!("Failed to lock database");
    };
    let value = database.get(key)?;
    Ok(OutboundMessage::Get(value))
}

fn handle_action_keys(
    database: &Arc<Mutex<Database>>,
    pattern: String,
) -> anyhow::Result<OutboundMessage> {
    let Ok(database) = database.lock() else {
        anyhow::bail!("Failed to lock database");
    };
    let value = database.keys(pattern)?;
    Ok(OutboundMessage::Keys(value))
}
