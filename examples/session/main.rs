use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    router::Next,
    server,
    session::cookie::new_session_manager
};
use serde::{
    Deserialize,
    Serialize
};

#[derive(Serialize, Deserialize)]
pub struct User {
    email: String,
    password: String,
}

pub fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.session().get("user_id") == "" {
        return res.redirect("register");
    }

    return next.handle(res);
}

pub fn guest<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.session().get("user_id") != "" {
        return res.redirect("/");
    }

    return next.handle(res);
}

pub async fn home_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html(format!("<h1>Welcome to protected home page user {}</h1>", req.session().get("user_id")).as_str());
}

pub async fn login<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.session().set("user_id", format!("{}", 1).as_str());

    return res.redirect("login");
}

pub async fn register<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html("<h1>Please visit the login page to login</h1>");
}

pub async fn logout<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.session().remove("user_id");

    return res.redirect("register");
}

pub async fn page_not_found<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html("<h1>404 Page Not Found</h1>");
}

fn main() {
    let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session", "encryption"));

    server.router().group("/", |router| {
        router.get("/", home_view, Some(vec![auth]));
        router.get("register", register, Some(vec![guest]));
        router.get("login", login, Some(vec![guest]));
        router.get("logout", logout, Some(vec![auth]));
        router.not_found(page_not_found);
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}