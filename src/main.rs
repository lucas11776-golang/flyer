use std::time::Duration;

use serde::{Deserialize, Serialize};

use flyer::{
    request::Request,
    response::Response,
    router::next::Next,
    server_tls,
    session::cookie::new_session_manager,
    validation::{Rules, Validator, rules}
};

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub token: String,
    pub r#type: String,
    pub expires: u128
}

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("login.html", None);
}

pub async fn login<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.json(&Token {
        token: String::from("eye.jwt.token"),
        r#type: String::from("jwt"),
        expires: Duration::from_hours(24).as_millis()
    });
}

async fn login_form<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    let mut rules = Rules::new();

    rules.field("email").add(rules::required, vec![]).add(rules::string, vec![]);
    rules.field("password").add(rules::required, vec![]).add(rules::string, vec![]);

    return Validator::handle(req, res, next, rules).await;
}

fn main() {
    let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"))
        .view("views")
        .assets("assets", 1024, Duration::from_hours(2).as_millis());

    server.router().group("/", |router| {
        router.get("/", index);
        router.group("register", |router| {
            router.post("/", login).middleware(login_form);
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}