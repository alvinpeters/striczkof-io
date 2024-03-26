mod listener;
mod tls;
mod utilities;
mod database;

use utilities::logging;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    logging::init(log::LevelFilter::Debug).expect("Failed to start the logger!");
    println!("Hell yeah!");
    Ok(())
}
