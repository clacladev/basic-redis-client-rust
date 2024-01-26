use self::config_message::ConfigMessage;
use std::time::SystemTime;

pub mod config_message;

const ID_CONFIG: &str = "CONFIG";
const ID_PING: &str = "PING";
const ID_ECHO: &str = "ECHO";
const ID_SET: &str = "SET";
const ID_GET: &str = "GET";
const ID_KEYS: &str = "KEYS";

#[derive(Debug)]
pub enum InboundMessage {
    Config(ConfigMessage),
    Ping,
    Echo(String),
    Set {
        key: String,
        value: String,
        expires_at: Option<u128>,
    },
    Get {
        key: String,
    },
    Keys {
        pattern: String,
    },
}

impl TryFrom<&[u8]> for InboundMessage {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let message_string = String::from_utf8_lossy(value).to_string();
        let lines: Vec<&str> = message_string
            .lines()
            .filter(|line| {
                *line == "$" || *line == "*" || (!line.starts_with("$") && !line.starts_with("*"))
            })
            .collect();

        if lines.is_empty() {
            anyhow::bail!(format!(
                "-> Failed to parse inbound message:\n'{}'",
                message_string
            ))
        }

        let message_id = lines[0].to_uppercase();
        match message_id.as_str() {
            ID_CONFIG => parse_config(&lines[1..]),
            ID_PING => parse_ping(),
            ID_ECHO => parse_echo(&lines[1..]),
            ID_SET => parse_set(&lines[1..]),
            ID_GET => parse_get(&lines[1..]),
            ID_KEYS => parse_keys(&lines[1..]),
            _ => anyhow::bail!(format!(
                "-> Failed to parse inbound message:\n'{}'",
                message_string
            )),
        }
    }
}

pub fn validate(lines: &[&str], min_length: usize, message_id: &str) -> anyhow::Result<()> {
    if lines.len() < min_length {
        anyhow::bail!("-> Failed to parse inbound message {message_id}")
    }
    Ok(())
}

fn get_option(lines: &[&str], key: &str) -> Option<String> {
    let key = key.to_lowercase();

    for (line_index, line) in lines.iter().enumerate() {
        if *line.to_lowercase() != key {
            continue;
        }
        let value_index = line_index + 1;
        if lines.len() < value_index {
            return None;
        }
        let value = lines[value_index];
        return Some(value.to_string());
    }

    None
}

fn parse_config(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 2, ID_CONFIG)?;
    let config_message = ConfigMessage::try_from(lines)?;
    Ok(InboundMessage::Config(config_message))
}

fn parse_ping() -> anyhow::Result<InboundMessage> {
    Ok(InboundMessage::Ping)
}

fn parse_echo(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 1, ID_ECHO)?;
    Ok(InboundMessage::Echo(lines[0].to_string()))
}

fn parse_set(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 2, ID_SET)?;
    let key = lines[0].to_string();
    let value = lines[1].to_string();

    let mut expires_at: Option<u128> = None;
    if let Some(expires_at_string) = get_option(&lines[2..], "PX") {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let expires_in = expires_at_string.parse::<u128>()?;
        expires_at = Some(now + expires_in);
    }

    Ok(InboundMessage::Set {
        key,
        value,
        expires_at,
    })
}

fn parse_get(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 1, ID_GET)?;
    let key = lines[0].to_string();
    Ok(InboundMessage::Get { key })
}

fn parse_keys(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 1, ID_KEYS)?;
    let pattern = lines[0].to_string();
    Ok(InboundMessage::Keys { pattern })
}
