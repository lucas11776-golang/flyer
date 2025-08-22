use std::io::Result;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
    first_name: String,
    last_name: String,
    email: String
}

#[tokio::main]
async fn main() -> Result<()> {
    // let mut server = flyer::server("127.0.0.1".to_string(), 9999).await?;
    let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert").await?;

    server.router().get("/api/users/{user}", |req, res| {
        return res.json(&User{
            id: req.parameter("user").parse().unwrap(),
            first_name: "Jeo".to_owned(),
            last_name: "Doe".to_owned(),
            email: "jeo@doe.com".to_owned()
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}