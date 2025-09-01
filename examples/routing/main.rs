use std::{io::Result};

use flyer::{request::Request, response::Response};

fn index<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

fn store<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

fn view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

fn update<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

fn destroy<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1", 9999).await?;
    
    server.router().group("api", |router| {
        router.group("users", |router| {
            router.get("/", index, None);
            router.post("/", store, None);
            router.group("{user}", |router| {
                router.get("/", view, None);
                router.patch("/", update, None);
                router.delete("/", destroy, None);
            }, None);
        }, None);
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}