use flyer::{server, request::Request, response::Response};

/// Advanced Routing Example
///
/// This example demonstrates:
/// - Grouping routes
/// - Using route parameters
/// - Handling different HTTP methods
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

pub async fn destroy(_req: Request, res: Response) -> Response {
    return res.redirect("users")
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

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
