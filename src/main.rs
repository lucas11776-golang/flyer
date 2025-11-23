use std::{fs::File, io::Write, time::Duration};

use flyer::{
    request::Request,
    response::Response,
    router::next::Next,
    server_tls,
    session::cookie::new_session_manager,
    validation::{Rules, Validator, rules},
    view::view_data
};

pub async fn home<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("index.html", Some(view_data()));
}

pub async fn upload<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    let uploaded = req.file("file").unwrap();
    let mut file = File::create(uploaded.name.as_str()).unwrap();

    file.write(&uploaded.content).unwrap();

    return res.redirect("/");
}

async fn register_form<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    let mut rules = Rules::new();

    rules.insert(String::from("file"), vec![(rules::required, vec![])]);

    return Validator::handle(req, res, next, rules).await;
}

fn main() {
    let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    // let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"))
        .view("views")
        .assets("assets", 1024, Duration::from_hours(2).as_millis());

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload).middleware(register_form);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}