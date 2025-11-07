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
    let mut server = flyer::server("127.0.0.1", 9999);

    server.router().ws("/", async |req, ws| {
        println!("Working on websocket");    



        ws.on( async |event, writer| {

            match event {
                flyer::ws::Event::Ready() => todo!(),
                flyer::ws::Event::Message(items) => {
                    println!("Message {:?}", String::from_utf8(items))
                },
                flyer::ws::Event::Ping(items) => todo!(),
                flyer::ws::Event::Pong(items) => todo!(),
                flyer::ws::Event::Close(reason) => todo!(),
            }

        });

    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}