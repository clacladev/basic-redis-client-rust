const ID_PING: &str = "PING";
const ID_ECHO: &str = "ECHO";
const ID_SET: &str = "SET";
const ID_GET: &str = "GET";

#[derive(Debug)]
pub enum InboundMessage {
    Ping,
    Echo(String),
    Set { key: String, value: String },
    Get { key: String },
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

fn parse_echo(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 2, ID_ECHO)?;
    Ok(InboundMessage::Echo(lines[1].to_string()))
}

fn parse_set(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 4, ID_SET)?;
    let key = lines[1].to_string();
    let value = lines[3].to_string();
    Ok(InboundMessage::Set { key, value })
}

fn parse_get(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    validate(lines, 2, ID_GET)?;
    let key = lines[1].to_string();
    Ok(InboundMessage::Get { key })
}
