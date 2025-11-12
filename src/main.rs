use std::time::Duration;

use flyer::{request::Request, response::Response, server, server_tls, session::cookie::new_session_manager, view::view_data};
use serde::{Deserialize, Serialize};

static ACCOUNTS: Vec<User> = vec![];

#[derive(Serialize, Deserialize)]
pub struct User {
    email: String,
    password: String,
}

pub async fn home_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    // req.session().set("user_id", format!("{}", 1).as_str());

    println!("user_id: {:?}", req.session().get("user_id"));

    return res.view("index.html", Some(view_data()));
}

pub async fn login_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("login.html", Some(view_data()));
}

pub async fn register_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("register.html", Some(view_data()));
}

pub async fn page_not_found<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("404.html", Some(view_data()));
}

fn main() {
    // let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    let mut server = server("127.0.0.1", 9999)
        // .assets("assets", 1024 * 10, (60 * 60) * 24)
        .assets("assets", 1024 * 1, 10)
        .view("views")
        .session(new_session_manager(Duration::from_hours(2), "session", "encryption"))
        ;

    server.router().group("/", |router| {
        router.get("/", home_view, None);
        router.group("register", |router| {
            router.get("/", register_view, None);
        }, None);
        router.group("login", |router| {
            router.get("/", login_view, None);
        }, None);
    }, None);

    server.router().not_found(page_not_found);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}