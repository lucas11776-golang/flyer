use flyer::{server};

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().ws("/", |req, ws| {
        ws.on_ready(|mut ws| async move {
            println!("Websocket connection is ready");
            ws.write("Hello Ready".into());
        });
        
        ws.on_message(|mut ws, data| async move {
            println!("Received message: {:?}", String::from_utf8(data.to_vec()).unwrap());
            ws.write("Hello message".into());
        });

        ws.on_ping(|mut ws, data| async move {
            println!("Received ping: {:?}", String::from_utf8(data.to_vec()).unwrap());
            ws.write("Hello ping".into());
        });

        ws.on_pong(|mut ws, data| async move {
            println!("Received pong: {:?}", String::from_utf8(data.to_vec()).unwrap());
            ws.write("Hello pong".into());
        });

        ws.on_close(|reason| async move {
            println!("Received close: {:?}", reason);
        });
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}