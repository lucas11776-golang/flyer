use std::{collections::HashMap, time::Duration};

use flyer::{
    request::{Request, form::Form},
    response::Response,
    router::next::Next,
    server,
    validation::{Rules}
};
use tokio::time::sleep;

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("register.html", None);
}

pub async fn register<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.with_flash("logged_in", "You have logged in successfully")
        .back();
}

pub async fn user_exists(table: &str, email: &str) -> bool {
    let mut db: HashMap<&str, Vec<&str>> = HashMap::new();

    db.insert("users", vec!["john@deo.com", "jane@deo.com"]);

    sleep(Duration::from_millis(250)).await;

    return db.get(table).unwrap_or(&Vec::new()).iter().find(|&u| u.eq(&email)).is_some();
}

pub async fn rule_email_exists(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    return if user_exists(&args[0], form.values.get(&field).unwrap_or(&String::new())).await {
        None
    } else {
        Some(String::from("The email does not exist"))
    };
}

async fn login_form<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    let mut rules = Rules::new();

    rules.rule("email", vec!["required", "string", "email_exists:users"])
        .rule("password", vec!["required", "string", "min:5", "max:21", "confirmed"]);

    return rules.handle(req, res, next);
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .view("views")
        .assets("assets", 1024, Duration::from_secs((60 * 60) * 2).as_millis());

    Rules::add("email_exists", rule_email_exists);

    server.router().group("/", |router| {
        router.get("/", index);
        router.post("register", register).middleware(login_form);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}