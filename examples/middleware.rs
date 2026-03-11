use flyer::{
    request::Request,
    response::Response,
    router::next::Next,
    server
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: u64,
    email:  String
}

#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
    message: String
}

pub async fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    if req.header("authorization") != "ey.jwt.token" {
        return res.status_code(401).json(&JsonMessage{
            message: "Unauthorized Access".to_owned()
        })
    }
    
    return next.handle(res);
}

pub async fn verified<'a>(_req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    // Some logic to check user in database
    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("api", |router| {
        router.group("users", |router| {
            router.group("{user}", |router| {
                router.get("/", async |req, res| {
                    return res.json(&User{
                        id: req.parameter("user").parse().unwrap(),
                        email: "joe@deo.com".to_owned()
                    });
                }).middleware(verified);
            });
        });
    }).middleware(auth);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}