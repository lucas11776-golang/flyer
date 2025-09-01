use std::{collections::HashMap, fs, io::Result};

use flyer::{request::Request, response::{Response, view_data}, router::Next, session::cookie::CookieSessionManager};
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

    let mut data = view_data();

    data.insert("first_name", "Jeo Deo");
    data.insert("age", &10);

    res.session().unwrap().set("user_id", "1");

    return res.view("index", Some(data));
}

fn home<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    let mut data = view_data();

    data.insert("user", &User {
        id: 1,
        first_name: "Themba".to_owned(),
        last_name: "Ngubeni".to_owned(),
        email: "themba@testing.com".to_owned(),
    });

    return res.view("index", Some(data));
}

fn services<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    // let data = view_data();
    return res.view("nested/services", None);
}

fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next<'a>) -> &'a mut Response {
    // return res.status_code(401).json(&JsonMessage{
    //     message: "unauthorized access".to_string()
    // });
    return next.next();
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1", 9999).await?;
    // let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert").await?;

    server.session(CookieSessionManager{token: "abc.test.token".to_string()})
        .view("views");

    server.router().get("/", home, None);
    server.router().get("/services", services, Some(vec![]));

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