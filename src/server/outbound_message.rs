use super::resp::{
    create_array_reply, create_bulk_strings_reply, create_null_bulk_strings_reply,
    create_simple_string_reply, NULL_BULK_STRING,
};

#[derive(Debug)]
pub enum OutboundMessage {
    Ok,
    ConfigGet { key: String, value: Option<String> },
    Pong,
    Echo(String),
    Get(Option<String>),
    Keys(Vec<String>),
}

impl Into<String> for OutboundMessage {
    fn into(self) -> String {
        match self {
            Self::Ok => create_simple_string_reply("OK"),
            Self::ConfigGet { key, value } => create_config_string(key, value),
            Self::Pong => create_simple_string_reply("PONG"),
            Self::Echo(string) => create_simple_string_reply(&string),
            Self::Get(None) => create_null_bulk_strings_reply(),
            Self::Get(Some(value)) => create_bulk_strings_reply(vec![value]),
            Self::Keys(values) => create_array_reply(values),
        }
    }
}

impl Into<Vec<u8>> for OutboundMessage {
    fn into(self) -> Vec<u8> {
        let string: String = self.into();
        string.into_bytes()
    }
}

fn create_config_string(key: String, value: Option<String>) -> String {
    let mut lines = vec![key];
    match value {
        Some(ref value) => lines.push(value.into()),
        None => lines.push(NULL_BULK_STRING.into()),
    }
    create_array_reply(lines)
}
