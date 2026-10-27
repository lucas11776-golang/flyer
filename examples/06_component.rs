use flyer::{server, view::{ViewData}};
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    first_name: &'static str,
    last_name: &'static str,
    email: &'static str
}

fn main() {
    // Expects a 'views' folder with 'index.html'
    let server = server("127.0.0.1", 9999)
        .view("views");

    // server.router().get("/", async |_req, res| {
    //     res.component()
    // });

    println!("Running Server: {}", server.address());

    server.listen();
}
