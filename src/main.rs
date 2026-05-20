use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    server, 
    session::cookie::SessionCookieManager,
    view::ViewData
};

pub async fn home<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("index.html", Some(ViewData::new()));
}

pub async fn upload<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    if req.file("file").is_none() {
        return res.with_error("file", "The file is required.")
            .back();
    }

    let save = req.file("file").unwrap().save("file").await.unwrap();
    let backup = req.file("file").unwrap().save_as("backup", "backup").await.unwrap();

    println!("SAVE PATH {}", save);
    println!("BACKUP PATH {}", backup);

    return res.redirect("/");
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .session(SessionCookieManager::new(Duration::from_secs((60 * 60) * 2), "session_cookie_key_name", "encryption"))
        .view("views")
        .set_request_max_size(1024 * 100); // Max Request size 100MB

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}