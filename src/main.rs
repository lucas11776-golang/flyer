use serde::{Deserialize, Serialize};

use flyer::{request::Request, response::Response, router::Next, view::view_data};

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    pub id: i64,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str
}

#[derive(Serialize, Deserialize)]
pub struct Message<'a> {
    message: &'a str
}

pub fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.header("authorization") != "jwt.token" {
        let writer = res.ws.as_mut().unwrap();

        writer.write(serde_json::to_vec(&Message{message: "Unauthorized Access"}).unwrap());
        
        return res;
    }

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

    server.router().group("/", |router| {
        router.get("/",   async |_req, res| {
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

        router.get("/api",   async |_req, res| {
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

    server.router().group("", |router| {
        router.ws("/", async |_req, ws| {
            ws.on( async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(_items) => writer.write("Hello This Public Route".into()),
                    flyer::ws::Event::Binary(_items) => todo!(),
                    flyer::ws::Event::Ping(_items) => todo!(),
                    flyer::ws::Event::Pong(_items) => todo!(),
                    flyer::ws::Event::Close(_reason) => todo!(),
                }
            });
        }, None);

        router.ws("/private", async |_req, ws| {
            ws.on( async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(_items) => writer.write("Hello This Private Route".into()),
                    flyer::ws::Event::Binary(_items) => todo!(),
                    flyer::ws::Event::Ping(_items) => todo!(),
                    flyer::ws::Event::Pong(_items) => todo!(),
                    flyer::ws::Event::Close(_reason) => todo!(),
                }
            });
        },Some(vec![auth]));
    }, None);


    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}