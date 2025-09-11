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
        ws.on_ready(|ws| async {
            println!("Websocket connection is ready");
        });

        ws.on_message(|ws, data| async move {
            println!("Received message: {:?}", String::from_utf8(data.to_vec()).unwrap());
        });

        ws.on_ping(|ws, data| async move {
            println!("Received ping: {:?}", String::from_utf8(data.to_vec()).unwrap());
        });

        ws.on_pong(|ws, data| async move {
            println!("Received pong: {:?}", String::from_utf8(data.to_vec()).unwrap());
        });

        ws.on_close(|reason| async move {
            println!("Received close: {:?}", reason);
        });
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}