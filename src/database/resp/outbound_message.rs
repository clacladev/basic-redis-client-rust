#[derive(Debug)]
pub enum OutboundMessage {
    Pong,
}

impl Into<String> for OutboundMessage {
    fn into(self) -> String {
        match self {
            Self::Pong => "+PONG\r\n".to_string(),
        }
    }
}

impl Into<Vec<u8>> for OutboundMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        string.into_bytes()
    }
}
