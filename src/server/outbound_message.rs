use super::resp::{
    create_array_reply, create_bulk_strings_reply, create_null_bulk_strings_reply,
    create_simple_string_reply, NULL_BULK_STRING,
};

#[derive(Debug)]
pub enum OutboundMessage {
    Ok,
    Pong,
    Echo(String),
    Get(Option<String>),
    ConfigGet { key: String, value: Option<String> },
}

impl Into<String> for OutboundMessage {
    fn into(self) -> String {
        match self {
            Self::Ok => create_simple_string_reply("OK"),
            Self::Pong => create_simple_string_reply("PONG"),
            Self::Echo(string) => create_simple_string_reply(&string),
            Self::Get(None) => create_null_bulk_strings_reply(),
            Self::Get(Some(value)) => create_bulk_strings_reply(vec![&value]),
            Self::ConfigGet { key, value } => create_config_string(key, value),
        }
    }
}

impl Into<Vec<u8>> for OutboundMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        println!("-> Outbound message encoded: {:?}", string);
        string.into_bytes()
    }
}

fn create_config_string(key: String, value: Option<String>) -> String {
    let mut lines = vec![key.as_str()];
    match value {
        Some(ref value) => lines.push(value),
        None => lines.push(NULL_BULK_STRING),
    }
    create_array_reply(lines)
}
