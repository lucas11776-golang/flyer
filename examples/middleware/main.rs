use std::{io::Result};

use flyer::{request::Request, response::Response, router::Next};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct User {
    id: u64,
    email:  String
}

#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
    message: String
}

pub fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next<'a>) -> &'a mut Response {
    return res.status_code(401).json(&JsonMessage{
        message: "unauthorized access".to_string()
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1", 9999).await?;
    
    server.router().get("api/users/{user}", |req, res| {
        return res.json(&User{
            id: req.parameter("user").parse().unwrap(),
            email: "joe@deo.com".to_owned()
        })
    }, Some(vec![auth]));

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}