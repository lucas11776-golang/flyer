use std::time::Duration;

use serde::{Deserialize, Serialize};

use flyer::{
    request::Request,
    response::Response,
    router::Next,
    server,
    session::cookie::new_session_manager,
    view::view_data
};

// static ACCOUNTS: Vec<User> = vec![];

#[derive(Serialize, Deserialize)]
pub struct User {
    email: String,
    password: String,
}

pub async fn home_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.cookies()
        .set("user_id", "1")
        .set_expires(Duration::from_hours(2));

    req.cookies()
        .set("tracker_id", "t_1_2")
        .set_expires(Duration::from_hours(2));


    return res.view("index.html", Some(view_data()));
}

pub async fn login_view<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    // req.session().set("user_id", "1");

    return res.view("login.html", Some(view_data()));
}

pub async fn register_view<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("register.html", Some(view_data()));
}

pub async fn page_not_found<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("404.html", Some(view_data()));
}


pub async fn auth<'a>(_req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    println!("AUTH");

    return next.handle(res);
}

pub async fn guest<'a>(_req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    println!("GUEST");

    return next.handle(res);
}

pub async fn csrf<'a>(_req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    println!("CSRF");

    return next.handle(res);
}

fn main() {
    // let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    let mut server = server("127.0.0.1", 9999)
        .assets("assets", 1024 * 10, (60 * 60) * 24)
        .assets("assets", 1024 * 1, 10)
        .view("views")
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"));

    server.router().group("/", |router| {
        router.get("/", home_view)
            .middleware(auth);
        router.group("register", |router| {
            router.get("/", register_view)
                .middleware(guest);
        });
        router.group("login", |router| {
            router.get("/", login_view)
                .middleware(auth);
        });
    });

    server.router().not_found(page_not_found);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}