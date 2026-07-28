use flyer::{
    request::Request,
    response::Response,
    server,
    utils::{env}
};

/// Environment Variables Example
///
/// This example demonstrates:
/// - Loading a .env file
/// - Accessing configuration using `env()`
/// - Using environment settings to configure the server
pub async fn index(_req: Request, res: Response) -> Response {
    return res.html("<h1>Check console for environment details</h1>");
}

fn main() {
    // Requires a .env file in the root
    env::load(".env");

    let host = env::env("HOST");
    let port: u32 = env::env("PORT").parse().unwrap_or(9999);

    let server = server(host, port)
        .view("views");

    server.router().group("/", |router| {
        router.get("/", index);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
