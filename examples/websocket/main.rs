use flyer::{server};

fn main() {
    let mut server = server("127.0.0.1", 9999);


    server.router().ws("/", |req, ws| {
        ws.on(|event| async {
            match event {
                flyer::ws::Event::Ready()                 => println!("Websocket connection is ready"),
                flyer::ws::Event::Message(items) => println!("Websocket connection is message: {:?}", String::from_utf8(items.to_vec()).unwrap()),
                flyer::ws::Event::Ping(items)    => println!("Websocket connection is ping: {:?}", String::from_utf8(items.to_vec()).unwrap()),
                flyer::ws::Event::Pong(items)    => println!("Websocket connection is pong: {:?}", String::from_utf8(items.to_vec()).unwrap()),
                flyer::ws::Event::Close(reason)   => println!("Websocket connection is close: {:?}", reason),
            }
        });
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}