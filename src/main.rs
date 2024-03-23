mod listener;
mod tls;
mod utilities;
mod database;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hell yeah!");
    Ok(())
}
