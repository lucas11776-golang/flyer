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


    server.router().ws("/", async |req, mut ws| {
        ws.on(async |event, mut writer| {
            match event {
                flyer::ws::Event::Ready() => writer.write("Ready".as_bytes().to_vec()).await,
                flyer::ws::Event::Message(items) => writer.write("Message".as_bytes().to_vec()).await,
                flyer::ws::Event::Ping(items) => writer.write("Ping".as_bytes().to_vec()).await,
                flyer::ws::Event::Pong(items) => writer.write("Pong".as_bytes().to_vec()).await,
                flyer::ws::Event::Close(reason) => todo!(),
            }
        });

    }, None);


    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}