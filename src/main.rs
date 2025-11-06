use serde::{Deserialize, Serialize};

use flyer::{request::Request, response::Response, router::Next, view::view_data};

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    pub id: i64,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str
}

pub fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    println!("AUTH ALL");

    return next.handle(res);
}

pub fn auth_web<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    println!("AUTH WEB");

    if req.is_json() {
        return res.status_code(400)
    }

    return next.handle(res);
}

pub fn auth_json<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    println!("AUTH JSON");

    if !req.is_json() {
        return res.status_code(400)
    }

    return next.handle(res);
}

fn main() {
    // let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert");
    let mut server = flyer::server("127.0.0.1", 9999)
        .view("views");


    server.router().group("/", |mut router| {
        router.get("/",   async |req, res| {
            let mut data = view_data();

            let user = User {
                id: 1,
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo@doe.com",
            };

            data.insert("user", &user);

            return res.view("index.html", Some(data));   
        }, Some(vec![auth_web]));

        router.get("/api",   async |req, res| {
            let mut data = view_data();

            let user = User {
                id: 1,
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo@doe.com",
            };

            data.insert("user", &user);

            return res.json(&user);   
        }, Some(vec![auth_json]));
    }, Some(vec![]));


    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}