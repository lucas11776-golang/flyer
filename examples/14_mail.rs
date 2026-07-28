use flyer::{mail::Mail, server};
use uuid::Uuid;

/// Mail Example
///
/// This example demonstrates:
/// - Configuring the mailer
/// - Sending a simple email
/// - Sending to multiple recipients
fn main() {
    let server = server("127.0.0.1", 9999)
        .mailer("127.0.0.1".to_string(), 5555, "".to_string(), "".to_string(), false);
    
    server.router().get("/send-mail", async |_req, res| {
        // Send to single email
        Mail::new()
            .from("no-reply@test.com".to_string(), Some("no-reply"))
            .html(format!("<h1>Token: {}</h1>", Uuid::new_v4()))
            .send("user@test.com".to_string(), Some("User"))
            .unwrap();

        return res.html("<h1>Email sent!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
