use self::resp::outbound_message::OutputMessage;
use crate::database::resp::inbound_message::InputMessage;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: u32 = 6379;
const MB: usize = 1024 * 1024;

mod resp;

pub fn start_database() -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", DEFAULT_IP, DEFAULT_PORT))?;
    println!("-> Server database at {}:{}", DEFAULT_IP, DEFAULT_PORT);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => handle_stream(&mut stream)?,
            Err(e) => println!("-> Error: {}", e),
        }
    }

    Ok(())
}

fn handle_stream(stream: &mut TcpStream) -> anyhow::Result<()> {
    let mut buffer = [0; 1 * MB];
    let bytes_read = stream.read(&mut buffer)?;
    let input_message = InputMessage::try_from(&buffer[..bytes_read])?;
    println!("-> Input message: {:?}", input_message);

    let output_message = handle_message(&input_message)?;
    println!("-> Output message: {:?}", output_message);
    let output_message_bytes: Vec<u8> = output_message.into();
    stream.write_all(&output_message_bytes)?;

    Ok(())
}

fn handle_message(message: &InputMessage) -> anyhow::Result<OutputMessage> {
    match message {
        &InputMessage::Ping => Ok(OutputMessage::Pong),
    }
}
