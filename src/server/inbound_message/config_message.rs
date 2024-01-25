use super::validate;

const ID_GET: &str = "GET";

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Get { key: String },
}

impl TryFrom<&[&str]> for ConfigMessage {
    type Error = anyhow::Error;

    fn try_from(lines: &[&str]) -> Result<Self, Self::Error> {
        let message_id = lines[0].to_uppercase();
        match message_id.as_str() {
            ID_GET => parse_get(&lines[1..]),
            _ => anyhow::bail!(format!(
                "-> Failed to parse inbound config message:\n'{:?}'",
                lines
            )),
        }
    }
}

fn parse_get(lines: &[&str]) -> anyhow::Result<ConfigMessage> {
    validate(lines, 1, ID_GET)?;
    Ok(ConfigMessage::Get {
        key: lines[0].to_string(),
    })
}
