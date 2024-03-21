mod listener;
mod tls;
mod utilities;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hell yeah!");
    Ok(())
}
