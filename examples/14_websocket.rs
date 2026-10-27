use flyer::{server, websocket::Websocket};

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("", |router| {
        router.ws("/", async |_req, ws| -> Websocket {
            ws
                .ready(async |_socket| {
                    todo!()
                })
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
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
