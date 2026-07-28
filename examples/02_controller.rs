use flyer::{request::Request, response::Response, server};


pub async fn index(_req: Request, res: Response) -> Response {
    res.html("<h1>Controller</h1>")
}

pub fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", index);

    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());

    server.listen();
}