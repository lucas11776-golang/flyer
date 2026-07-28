use flyer::server;

pub fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", async |_req, res| {
        res
    });

    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());

    server.listen();
}