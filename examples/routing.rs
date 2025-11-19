use flyer::{server, request::Request, response::Response};

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html("<h1>Users List</h1>");
}

pub async fn store<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.redirect("users/1");
}

pub async fn view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html(format!("<h1>User {}</h1>", req.parameter("user")).as_str());
}

pub async fn update<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.redirect(format!("users/{}", req.parameter("user")).as_str());
}

pub async fn destroy<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.redirect("users")
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
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