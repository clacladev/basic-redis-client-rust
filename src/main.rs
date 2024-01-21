use database::start_database;

mod database;

fn main() {
    // let args: Vec<String> = env::args().collect();
    // let Ok(options) = CliOption::from_str(&args[1..]) else {
    //     panic!("Failed to parse options");
    // };

    match start_database() {
        Ok(_) => {}
        Err(e) => println!("-> Error: {}", e),
    }
}
