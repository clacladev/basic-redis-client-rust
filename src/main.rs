use database::start_database;

mod database;

#[tokio::main]
async fn main() {
    // let args: Vec<String> = env::args().collect();
    // let Ok(options) = CliOption::from_str(&args[1..]) else {
    //     panic!("Failed to parse options");
    // };

    match start_database().await {
        Ok(_) => {}
        Err(e) => println!("-> Error: {}", e),
    }
}
