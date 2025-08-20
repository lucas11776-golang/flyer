use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1".to_string(), 9999).await?;

    server.router().get("/".to_owned(), |_req, res| {
        return res.html("<h1>Hello World!!!</h1>".to_owned());
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}