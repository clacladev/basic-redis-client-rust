#[derive(Debug)]
pub enum OutboundMessage {
    Pong,
    Echo(String),
}

impl Into<String> for OutboundMessage {
    fn into(self) -> String {
        match self {
            Self::Pong => "+PONG\r\n".to_string(),
            Self::Echo(string) => format!("+{}\r\n", string),
        }
    }
}

impl Into<Vec<u8>> for OutboundMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        string.into_bytes()
    }
}
