use std::{collections::HashMap, fs, io::Result};

use flyer::{request::Request, response::{Response, ViewData}, router::Next, session::DefaultSessionManager};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
    first_name: String,
    last_name: String,
    email: String
}

#[derive(Serialize, Deserialize)]
struct JsonMessage {
    message: String
}

fn view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    let file = req.file("profile").unwrap();
    
    fs::write(file.name.clone(), file.content.clone()).unwrap();

    println!("{:?}", req.values.get("first_name").unwrap());


    let mut data: ViewData = HashMap::new();

    data.insert("first_name".to_string(), Box::new("Jeo Deo"));
    data.insert("age".to_string(), Box::new(10));


    res.session.as_ref().unwrap().set("user_id", "1");


    return res.view("index", data);
}

fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next<'a>) -> &'a mut Response {
    return res.status_code(401).json(&JsonMessage{
        message: "unauthorized access".to_string()
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    // let mut server = flyer::server("127.0.0.1", 9999).await?;
    let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert").await?;

    // server.session("abc.session.key");

    // server.session(DefaultSessionManager::new("abc.test.token"));
    server.session(DefaultSessionManager{token: "abc.test.token".to_string()});

    server.router().group("api", |router| {
        router.group("users", |router| {
            router.group("{user}", |router| {
                router.post("/", view,  None);
                router.get("/", view,  None);
            }, None);
        }, None);
    }, Some(vec![auth]));

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}