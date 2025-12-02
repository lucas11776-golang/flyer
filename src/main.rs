use std::time::Duration;

use flyer::{
    request::Request, response::Response, server, session::cookie::new_session_manager, utils::{env, load_env}
};

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("env.html", None);
}

fn main() {
    load_env(".env");

    let mut server = server(env("HOST").as_str(), env("PORT").parse().unwrap())
        .session(new_session_manager(Duration::from_hours(2), "cookie_token", "test_123"))
        .view("views");

    server.router().group("/", |router| {
        router.get("/", index);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}