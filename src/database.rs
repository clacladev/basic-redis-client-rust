use self::resp::outbound_message::OutboundMessage;
use crate::database::resp::inbound_message::InboundMessage;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: u32 = 6379;
const MB: usize = 1024 * 1024;

mod actions;
mod resp;

pub async fn start_database() -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", DEFAULT_IP, DEFAULT_PORT)).await?;
    println!("-> Server database at {}:{}", DEFAULT_IP, DEFAULT_PORT);

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move { handle_stream(&mut stream).await });
    }
}

async fn handle_stream(stream: &mut TcpStream) -> anyhow::Result<()> {
    loop {
        let mut buffer: Vec<u8> = Vec::with_capacity(1 * MB);
        let bytes_read = stream.read_buf(&mut buffer).await?;
        let inbound_message = InboundMessage::try_from(&buffer[..bytes_read])?;
        println!("-> Inbound message: {:?}", inbound_message);

        let outbound_message = handle_message(&inbound_message)?;
        println!("-> Outbound message: {:?}", outbound_message);
        let outbound_message_bytes: Vec<u8> = outbound_message.into();
        stream.write_all(&outbound_message_bytes).await?;
    }
}

fn handle_message(message: &InboundMessage) -> anyhow::Result<OutboundMessage> {
    match message {
        &InboundMessage::Ping => Ok(OutboundMessage::Pong),
        &InboundMessage::Echo(ref string) => Ok(OutboundMessage::Echo(string.clone())),
        &InboundMessage::Set { ref key, ref value } => {
            actions::set(key.as_str(), value.as_str())?;
            Ok(OutboundMessage::Ok)
        }
    }
}
