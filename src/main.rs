use flyer::{server, view::{ViewData, render_view}};
use serde::Serialize;

#[derive(Serialize)]
pub struct User<'a> {
    first_name: &'a str,
    last_name: &'a str,
    email: &'a str
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .view("views");


    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            let mut data = ViewData::new();

            data.insert("user", &User{
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo.deo@gmail.com",
            });

            return res.view("index.html", Some(data));
        });
        router.get("/render", async |_req, res| {
            let user = User{
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo.deo@gmail.com",
            };

            // This helper function is useful when sending email`s etc.
            let html = render_view("render.html", Some(ViewData::with("user", &user))).unwrap();

            return res.html(&html);
        });
    });

    println!("Running Server: {}", server.address());

    server.listen();
}