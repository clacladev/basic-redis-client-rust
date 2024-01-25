pub fn create_simple_string_reply(string: &str) -> String {
    format!("+{}\r\n", string)
}

pub fn create_null_bulk_strings_reply() -> String {
    "$-1\r\n".to_string()
}

pub fn create_bulk_strings_reply(string: &str) -> String {
    format!("${}\r\n{}\r\n", string.len(), string)
}
