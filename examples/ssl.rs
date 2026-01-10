use flyer::server_tls;

fn main() {
    let mut server = server_tls("127.0.0.1", 9999, "host.cert", "host.key");
    
    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}