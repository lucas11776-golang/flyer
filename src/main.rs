
use std::time::Duration;

use flyer::{server, websocket::Websocket};
use tokio::time::sleep;

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().ws("/", async |_req, ws| -> Websocket {
        let socket = ws.socket().clone();

        // Ping client every 5 seconds
        let ping_handler = tokio::spawn(async move {
            loop {
                println!("Pinging Client Every 60 seconds");

                sleep(Duration::from_secs(5)).await;

                socket.ping(Default::default()).await.unwrap();
            }
        });

        ws
            // We want to value to stay alive even if we are out of scope you may add as many as you like.
            .keep_alive(ping_handler)
            .text(async |_payload, _socket| {
                todo!()
            })
            .binary(async |_payload, _socket| {
                todo!()
            })
            .ping(async |_payload, _socket| {
                todo!()
            })
            .pong(async |_payload, _socket| {
                todo!()
            })
            .close(async |_reason| {
                todo!()
            })
    });

    server.listen();
}