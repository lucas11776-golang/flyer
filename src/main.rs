use serde::{Deserialize, Serialize};

use flyer::ws::Event;

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    pub id: i64,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str
}

fn main() {
    // let mut server = flyer::server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    let mut server = flyer::server("127.0.0.1", 9999)
        .view("views");

    server.router().ws("/", async |req, mut ws| {
        ws.on(async |event, mut writer| {
            match event {
                Event::Ready() => writer.write("Ready".as_bytes().to_vec()).await,
                Event::Message(items) => writer.write("Message".as_bytes().to_vec()).await,
                Event::Ping(items) => writer.write("Ping".as_bytes().to_vec()).await,
                Event::Pong(items) => writer.write("Pong".as_bytes().to_vec()).await,
                Event::Close(reason) => todo!(),
            }
        });
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}