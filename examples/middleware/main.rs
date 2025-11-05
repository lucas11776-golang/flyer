use flyer::{server, request::Request, response::Response, router::Next};
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

pub fn auth<'a>(_req: &'a mut Request, res: &'a mut Response, _next: Next<'a>) -> &'a mut Response {
    // return res.status_code(401).json(&JsonMessage{
    //     message: "unauthorized access".to_string()
    // });


    return res;
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().get("api/users/{user}", async |req, res| {
        return res.json(&User{
            id: req.parameter("user").parse().unwrap(),
            email: "joe@deo.com".to_owned()
        })
    }, Some(vec![]));

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}