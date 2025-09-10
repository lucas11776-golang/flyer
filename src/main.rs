use flyer::{request::Request, server_tls, view::view_data, ws::Ws};


pub fn ws<'a>(req: &'a mut Request, ws: &'a mut Ws) {


    println!("Temp fix");

     ws.on_ready(|ws| async {

        // ws.write(vec![1, 3, 4]).;

        
        println!("Ready...");
        // ws.on_message(|ws, data| {
        //     ws.write_string("Hello World".to_owned()).unwrap();
        //     println!("Received data: {:?}", data);
        // });

        // ws.on_message(|ws, data| async {

        // });
    });
}

fn main() {
    // let mut server = server_tls("127.0.0.1", 9999, "host.key", "host.cert");
    let mut server = flyer::server("127.0.0.1", 9999);

    server.view("views");


    // server.router()
    
    server.router().get("api/users/{user}", |req, res| {
        let mut data = view_data();

        data.insert("first_name", "Jeo");
        data.insert("last_name", "Doe");
        data.insert("email", "jeo@doe.com");
        data.insert("age", &23);

        return res.view("index.html", Some(data))
    }, None);


    server.router().ws("/", ws, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}