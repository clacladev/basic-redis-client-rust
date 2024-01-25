use super::resp::{
    create_bulk_strings_reply, create_null_bulk_strings_reply, create_simple_string_reply,
};

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
            Self::Ok => create_simple_string_reply("OK"),
            Self::Pong => create_simple_string_reply("PONG"),
            Self::Echo(string) => create_simple_string_reply(&string),
            Self::Get(None) => create_null_bulk_strings_reply(),
            Self::Get(Some(value)) => create_bulk_strings_reply(&value),
        }
    }
}

impl Into<Vec<u8>> for OutboundMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        string.into_bytes()
    }
}
