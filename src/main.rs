use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut server = flyer::server("127.0.0.1".to_string(), 9999).await?;
    let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert").await?;

    // TODO: Fix router match...
    server.router().get("/", |req, res| {
        return res.html("<h1>Hello World</h1>");
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}