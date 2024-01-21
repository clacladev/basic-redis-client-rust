use std::net::TcpListener;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: u32 = 6379;

pub fn start_database() -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", DEFAULT_IP, DEFAULT_PORT))?;
    println!("-> Server database at {}:{}", DEFAULT_IP, DEFAULT_PORT);

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
