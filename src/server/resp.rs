pub const END_OF_LINE: &str = "\r\n";
pub const NULL_BULK_STRING: &str = "$-1";

pub fn create_simple_string_reply(string: &str) -> String {
    format!("+{string}{END_OF_LINE}")
}

pub fn create_null_bulk_strings_reply() -> String {
    format!("{NULL_BULK_STRING}{END_OF_LINE}")
}

pub fn create_bulk_strings_reply(lines: Vec<&str>) -> String {
    let mut reply = String::new();
    for line in lines {
        reply.push_str(&format!("${}{END_OF_LINE}{line}{END_OF_LINE}", line.len()));
    }
    reply
}

pub fn create_array_reply(lines: Vec<&str>) -> String {
    format!(
        "*{}{END_OF_LINE}{}",
        lines.len(),
        create_bulk_strings_reply(lines)
    )
}
