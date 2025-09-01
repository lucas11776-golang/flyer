use std::{io::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1", 9999).await?;
    
    server.router().get("", |req, res| {
        return res
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}