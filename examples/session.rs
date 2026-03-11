use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    router::next::Next,
    server,
    session::cookie::new_session_manager
};

/// Controller
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

/// Middleware
pub async fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.session().get("user_id") == "" {
        return res.redirect("register");
    }

    return next.handle(res);
}

pub async fn guest<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.session().get("user_id") != "" {
        return res.redirect("/");
    }

    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"));

    server.router().group("/", |router| {
        router.get("/", home_view).middleware(auth);
        router.get("register", register).middleware(guest);
        router.get("login", login).middleware(guest);
        router.get("logout", logout).middleware(auth);
    });

    server.router().not_found(page_not_found);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}