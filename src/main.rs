use flyer::{server, server_tls, session::cookie::new_session_cookie, view::view_data};

fn main() {
    let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    // let mut server = server("127.0.0.1", 9999)
        // .assets("assets", 1024 * 10, (60 * 60) * 24)
        .assets("assets", 1024 * 1, 10)
        .view("views")
        .session(new_session_cookie());
    
    server.router().get("/", async |_req, res| {
        return res.view("index.html", Some(view_data()));
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}