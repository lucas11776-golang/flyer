use flyer::{request::Request, response::Response, server};

async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("index.html", None);
}

fn main() {
    // let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    let mut server = server("127.0.0.1", 9999)
        .view("views");

    // println!("NUMBER -> {}", );

    server.router().group("/", |router| {
        router.get("/", index);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}