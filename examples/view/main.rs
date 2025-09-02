use std::{io::Result};

use flyer::view::{view_data};

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1", 9999).await?;

    // Create view folder in base project directory.
    server.view("views");
    
    server.router().get("/", |req, res| {
        let mut data = view_data();

        data.insert("first_name", "Jeo");
        data.insert("last_name", "Doe");
        data.insert("email", "jeo@doe.com");
        data.insert("age", &23);

        // Create file called index.html in views folder.
        return res.view("index.html", Some(data))
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}