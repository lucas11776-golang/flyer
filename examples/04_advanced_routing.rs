use flyer::{
    error::Error,
    request::Request,
    response::{HTTP_NOT_FOUND, Response},
    routing::next::Next,
    server
};

pub async fn index(_req: Request, res: Response) -> Response {
    return res.html("<h1>Users List</h1>");
}

pub async fn store(_req: Request, res: Response) -> Response {
    return res.redirect("users/1");
}

pub async fn view(req: Request, res: Response) -> Response {
    return res.html(format!("<h1>User {}</h1>", req.parameter("user")).as_str());
}

pub async fn update(req: Request, res: Response) -> Response {
    return res.redirect(format!("users/{}", req.parameter("user")).as_str());
}

// When calling destroy error controller will be called because get_user_id is `None`
pub async fn destroy(_req: Request, res: Response) -> Response {
    let get_user_id: Option<String> = None;

    get_user_id.unwrap();

    return res.redirect("users")
}

pub async fn not_found(_req: Request, res: Response) -> Response {
    res
        .status_code(HTTP_NOT_FOUND)
        .html("<h1>Hello World!!!</h1>")
}

pub async fn error(_error: Error, _req: Request, res: Response, _next: Next) -> Response {
    res
        .status_code(HTTP_NOT_FOUND)
        .html("<h1>500 Internal Server Error</h1>")
}

fn main() {
    let server = server("127.0.0.1", 9999);
    
    server.router().group("/", |router| {
        router.group("users", |router| {
            router.get("/", index);
            router.post("/", store);
            router.group("{user}", |router| {
                router.get("/", view);
                router.patch("/", update);
                router.delete("/", destroy);
            });
        });
    });

    server.router().not_found(not_found);

    server.error(error);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}