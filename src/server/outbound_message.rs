#[derive(Debug)]
pub enum OutboundMessage {
    Ok,
    Pong,
    Echo(String),
    Get(Option<String>),
}

impl Into<String> for OutboundMessage {
    fn into(self) -> String {
        match self {
            Self::Ok => "+OK\r\n".to_string(),
            Self::Pong => "+PONG\r\n".to_string(),
            Self::Echo(string) => format!("+{}\r\n", string),
            Self::Get(None) => "$0\r\n\r\n".to_string(),
            Self::Get(Some(value)) => format!("${}\r\n{}\r\n", value.len(), value),
        }
    }
}

impl Into<Vec<u8>> for OutboundMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        string.into_bytes()
    }
}
