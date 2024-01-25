const END_OF_LINE: &str = "\r\n";

fn create_reply(lines: Vec<&str>) -> String {
    let mut reply = String::new();
    for line in lines {
        reply.push_str(&line);
        reply.push_str(END_OF_LINE);
    }
    reply
}

pub fn create_simple_string_reply(string: &str) -> String {
    create_reply(vec![format!("+{}", string).as_str()])
}

pub fn create_null_bulk_strings_reply() -> String {
    create_reply(vec!["$-1"])
}

pub fn create_bulk_strings_reply(string: &str) -> String {
    let length_string = format!("${}", string.len());
    create_reply(vec![&length_string, string])
}
