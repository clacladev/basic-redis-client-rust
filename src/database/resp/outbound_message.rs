#[derive(Debug)]
pub enum OutputMessage {
    Pong,
}

impl Into<String> for OutputMessage {
    fn into(self) -> String {
        match self {
            Self::Pong => "+PONG\r\n".to_string(),
        }
    }
}

impl Into<Vec<u8>> for OutputMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        string.into_bytes()
    }
}
