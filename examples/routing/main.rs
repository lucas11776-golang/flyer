use flyer::{server, request::Request, response::Response};

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn store<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn view<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn update<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn destroy<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().group("api", |mut router| {
        router.group("users", |mut router| {
            router.get("/", index, None);
            router.post("/", store, None);
            router.group("{user}", |mut router| {
                router.get("/", view, None);
                router.patch("/", update, None);
                router.delete("/", destroy, None);
            }, None);
        }, None);
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}