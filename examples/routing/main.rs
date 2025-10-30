use flyer::{server, request::Request, response::Response};

pub async fn index(_req: Request, res: Response) -> Response {
    return res
}

pub async fn store(_req: Request, res: Response) -> Response {
    return res
}

pub async fn view(_req: Request, res: Response) -> Response {
    return res
}

pub async fn update(_req: Request, res: Response) -> Response {
    return res
}

pub async fn destroy(_req: Request, res: Response) -> Response {
    return res
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().group("api", |mut router| {

        // router.get("/", async |req, res| {
        //     return res
        // }, Some(vec![]));

        router.group("users", |mut router| {
            router.get("/", index, None);
            router.post("/", store, None);
            router.group("{user}", |mut router| {
                // router.get("/", view, None);
                router.patch("/", update, None);
                router.delete("/", destroy, None);
            }, None);
        }, None);
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}