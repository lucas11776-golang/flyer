use flyer::server_tls;

fn main() {
    let server = server_tls("127.0.0.1", 9999, "host.key", "host.cert");
    
    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}