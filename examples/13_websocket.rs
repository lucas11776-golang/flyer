use flyer::{server, websocket::Websocket};

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("", |router| {
        router.ws("/", async |_req, ws| -> Websocket {
            ws.on(async |event, socket| {
                match event {
                    flyer::websocket::Event::Ready() => todo!(),
                    flyer::websocket::Event::Text(bytes) => {
                        println!("Received: {}", String::from_utf8_lossy(&bytes));
                        socket
                            .write("Hello from WebSocket!".into())
                            .await
                            .unwrap();
                    },
                    flyer::websocket::Event::Binary(_bytes) => todo!(),
                    flyer::websocket::Event::Ping(_bytes) => todo!(),
                    flyer::websocket::Event::Pong(_bytes) => todo!(),
                    flyer::websocket::Event::Close(_reason) => todo!(),
                }
            })
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
