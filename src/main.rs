use cli::CliParam;
use server::start_database;
use std::env;
mod cli;
mod database;
mod server;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let cli_params = CliParam::from(&args[1..]);

    match start_database(cli_params).await {
        Ok(_) => {}
        Err(e) => println!("-> Error: {}", e),
    }
}
