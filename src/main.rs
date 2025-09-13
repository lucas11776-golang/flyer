use flyer::{view::view_data};

fn main() {
    let mut server = flyer::server("127.0.0.1", 9999);

    server.view("views");
    
    server.router().get("api/users/{user}", |req, res| {
        let mut data = view_data();

        data.insert("first_name", "Jeo");
        data.insert("last_name", "Doe");
        data.insert("email", "jeo@doe.com");
        data.insert("age", &23);

        return res.view("index.html", Some(data))
    }, None);
    

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