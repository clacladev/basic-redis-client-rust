use std::time::SystemTime;

const ID_PING: &str = "PING";
const ID_ECHO: &str = "ECHO";
const ID_SET: &str = "SET";
const ID_GET: &str = "GET";

#[derive(Debug)]
pub enum InboundMessage {
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
}

impl TryFrom<&[u8]> for InboundMessage {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let message_string = String::from_utf8_lossy(value).to_string();
        let lines: Vec<&str> = message_string.lines().collect();

        if lines.len() < 3 {
            anyhow::bail!(format!(
                "-> Failed to parse inbound message:\n'{}'",
                message_string
            ))
        }

        let message_id = lines[2].to_uppercase();
        match message_id.as_str() {
            ID_PING => Ok(Self::Ping),
            ID_ECHO => parse_echo(&lines[3..]),
            ID_SET => parse_set(&lines[3..]),
            ID_GET => parse_get(&lines[3..]),
            _ => anyhow::bail!(format!(
                "-> Failed to parse inbound message:\n'{}'",
                message_string
            )),
        }
    }
}

fn validate(lines: &[&str], min_length: usize, message_id: &str) -> anyhow::Result<()> {
    if lines.len() < min_length {
        anyhow::bail!("-> Failed to parse inbound message {message_id}")
    }
    Ok(())
}

fn get_option(lines: &[&str], key: &str) -> Option<String> {
    let key = key.to_lowercase();

    for (line_index, line) in lines.iter().enumerate() {
        if line.starts_with("$") || *line.to_lowercase() != key {
            continue;
        }
        let value_index = line_index + 2;
        if lines.len() < value_index {
            return None;
        }
        let value = lines[value_index];
        return Some(value.to_string());
    }

    None
}

fn parse_echo(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 2, ID_ECHO)?;
    Ok(InboundMessage::Echo(lines[1].to_string()))
}

fn parse_set(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 4, ID_SET)?;
    let key = lines[1].to_string();
    let value = lines[3].to_string();

    let mut expires_at: Option<u128> = None;
    if let Some(expires_at_string) = get_option(&lines[4..], "PX") {
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
    validate(lines, 2, ID_GET)?;
    let key = lines[1].to_string();
    Ok(InboundMessage::Get { key })
}
