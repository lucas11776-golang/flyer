use flyer::server;

/// Basic Routing Example
/// 
/// This example demonstrates the most basic functionality:
/// - Initializing a server
/// - Defining a simple GET route
/// - Returning a basic HTML response
fn main() {
    let server = server("127.0.0.1", 9999);
    
    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
