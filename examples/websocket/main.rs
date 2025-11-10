use flyer::{server};
use flyer::{request::Request, response::Response, router::Next};
use serde::{Deserialize, Serialize};

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

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("", |mut router| {
        router.ws("/", async |req, ws| {
            ws.on( async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(items) => writer.write("Hello This Public Route".into()),
                    flyer::ws::Event::Binary(items) => todo!(),
                    flyer::ws::Event::Ping(items) => todo!(),
                    flyer::ws::Event::Pong(items) => todo!(),
                    flyer::ws::Event::Close(reason) => todo!(),
                }
            });
        }, None);

        router.ws("/private", async |req, ws| {
            ws.on( async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(items) => writer.write("Hello This Private Route".into()),
                    flyer::ws::Event::Binary(items) => todo!(),
                    flyer::ws::Event::Ping(items) => todo!(),
                    flyer::ws::Event::Pong(items) => todo!(),
                    flyer::ws::Event::Close(reason) => todo!(),
                }
            });
        },Some(vec![auth]));
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}