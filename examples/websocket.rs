use flyer::router::next::Next;
use flyer::{server};
use flyer::{request::Request, response::Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<'a> {
    message: &'a str
}

pub async fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.header("authorization") != "jwt.token" {
        return res;
    }

    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("", |router| {
        router.ws("/", async |_req, ws| {
            ws.on(async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(_items) => writer.write("Hello This Public Route".into()),
                    flyer::ws::Event::Binary(_items) => todo!(),
                    flyer::ws::Event::Ping(_items) => todo!(),
                    flyer::ws::Event::Pong(_items) => todo!(),
                    flyer::ws::Event::Close(_reason) => todo!(),
                }
            });
        });

        router.ws("private", async |_req, ws| {
            ws.on(async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(_items) => writer.write("Hello This Private Route".into()),
                    flyer::ws::Event::Binary(_items) => todo!(),
                    flyer::ws::Event::Ping(_items) => todo!(),
                    flyer::ws::Event::Pong(_items) => todo!(),
                    flyer::ws::Event::Close(_reason) => todo!(),
                }
            });
        }).middleware(auth);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}