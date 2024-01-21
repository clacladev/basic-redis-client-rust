const ID_PING: &str = "PING";
const ID_ECHO: &str = "ECHO";

#[derive(Debug)]
pub enum InboundMessage {
    Ping,
    Echo(String),
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
            ID_PING => return Ok(Self::Ping),
            ID_ECHO => return parse_echo_message(&lines[3..]),
            _ => anyhow::bail!(format!(
                "-> Failed to parse inbound message:\n'{}'",
                message_string
            )),
        }
    }
}

fn parse_echo_message(lines: &[&str]) -> anyhow::Result<InboundMessage> {
    if lines.len() < 2 {
        anyhow::bail!("-> Failed to parse inbound message Echo")
    }
    Ok(InboundMessage::Echo(lines[1].to_string()))
}
