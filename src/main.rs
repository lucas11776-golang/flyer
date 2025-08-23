use std::{io::Result};

use flyer::{request::Request, response::Response, router::Next};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
    first_name: String,
    last_name: String,
    email: String
}

fn view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.json(&User{
        id: req.parameter("user").parse().unwrap(),
        first_name: "Jeo".to_owned(),
        last_name: "Doe".to_owned(),
        email: "jeo@doe.com".to_owned()
    });
}

fn user_exist<'a>(req: &'static mut Request, res: &'static mut Response, next: &'a mut Next<'a>) -> &'a mut Response {
    return next.next();
}

#[tokio::main]
async fn main() -> Result<()> {
    // let mut server = flyer::server("127.0.0.1", 9999).await?;
    let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert").await?;

    server.router().group("api", |router| {
        router.group("users", |router| {
            router.group("{user}", |router| {
                router.get("/", view,  Some(vec![]));
            });
        });

    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}