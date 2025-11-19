use std::{fs::File, io::Write, time::Duration};

use flyer::{
    request::Request, response::Response, server, server_tls, session::cookie::new_session_manager, view::view_data
};

pub async fn home<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("index.html", Some(view_data()));
}

pub async fn upload<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    if req.file("file").is_none() {
        return res.with_error("file", "The file is required.")
            .back();
    }

    let uploaded = req.file("file").unwrap();
    let mut file = File::create(uploaded.name.as_str()).unwrap();

    file.write(&uploaded.content).unwrap();

    return res.redirect("/");
}

fn main() {
    let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    // let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"))
        .view("views");

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}