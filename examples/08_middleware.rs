use flyer::{
    request::Request, response::{HTTP_UNAUTHORIZED, Response}, routing::next::Next, server
};
use serde::{Deserialize, Serialize};

/// Middleware Example
///
/// This example demonstrates:
/// - Creating middleware functions
/// - Intercepting requests
/// - Stopping request propagation (e.g., for unauthorized access)
/// - Calling `next.handle()` to continue processing
#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
    message: String
}

pub async fn auth(req: Request, res: Response, next: Next) -> Response {
    // Basic check for an authorization header
    if req.header("authorization") != "my-secret-token" {
        return res.status_code(HTTP_UNAUTHORIZED).json(&JsonMessage{
            message: "Unauthorized Access".to_owned()
        })
    }
    
    // Continue processing the request
    return next.handle(req, res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("api", |router| {
        router.get("/", async |_req, res| {
            return res.html("<h1>Authorized Access</h1>");
        });
    }).middleware(auth);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
