use std::{fs::File, io::Write};

use flyer::{view::view_data};
use serde::{Deserialize, Serialize};

// TODO: take rest must make controller route to support async operation...

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    pub id: i64,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str
}

fn main() {
    let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert")
        .view("views");

    server.router().get("/",   async |req, res| {
        let mut data = view_data();

        let user = User {
            id: 1,
            first_name: "Jeo",
            last_name: "Deo",
            email: "jeo@doe.com",
        };

        data.insert("user", &user);

        if let Some(image) =  req.file("image") {
            File::create(format!("test.png")).unwrap().write(&image.content).unwrap();
        }

        return res.view("index.html", Some(data));   
    }, None);

    server.router().group("api", |mut router| {
        router.group("users", |mut router| {
            router.get("{id}", async |req, res| {
                return res.json(&User {
                    id: req.parameters.get("id").unwrap().parse().unwrap(),
                    first_name: "Joe",
                    last_name: "Doe",
                    email: "jeo@doe.com",
                })
            }, None);
        }, None);
    }, None);

    server.router().not_found(async |_req, res| {
        return res.view("404.html", None)
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}