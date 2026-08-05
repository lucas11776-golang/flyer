use flyer::{
    request::Request,
    response::{HTTP_NOT_FOUND, Response},
    server
};

pub async fn index(_req: Request, res: Response) -> Response {
    res.html("<h1>Hello World!!!</h1>")
}

pub async fn not_found(_req: Request, res: Response) -> Response {
    res
        .status_code(HTTP_NOT_FOUND)
        .html("<h1>Hello World!!!</h1>")
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            return res.html("<h1>Hello World!!!</h1>")
        });
    });

    server.router().not_found(not_found);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
