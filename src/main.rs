use std::{fs::File, io::Write};

use flyer::{view::view_data};

// TODO: take rest must make controller route to support async operation...


// start: 'static
fn main() {
    let mut server = flyer::server("127.0.0.1", 9999);

    server.view("views");


    server.router().get("/",   async|req, res| {
        let mut data = view_data();

        data.insert("first_name", "Jeo");
        data.insert("last_name", "Doe");
        data.insert("email", "jeo@doe.com");
        data.insert("age", &23);

        if let Some(image) =  req.file("image") {
            File::create(format!("test.png")).unwrap().write(&image.content).unwrap();
        }

        return res.view("index.html", Some(data));   
    }, None);
    

    // server.router().ws("/", |req, ws| {
    //     tokio_scoped::scope(|scope| {
    //         scope.spawn(async {
    //             ws.write("Hello World".into()).await;
    //         });
    //     });

    //     ws.on(|event| async move {


    //         match event {
    //             flyer::ws::Event::Ready()                 => println!("Websocket connection is ready"),
    //             flyer::ws::Event::Message(items) => {


    //                 println!("Websocket connection is message: {:?}", String::from_utf8(items.to_vec()).unwrap())
    //             },
    //             flyer::ws::Event::Ping(items)    => println!("Websocket connection is ping: {:?}", String::from_utf8(items.to_vec()).unwrap()),
    //             flyer::ws::Event::Pong(items)    => println!("Websocket connection is pong: {:?}", String::from_utf8(items.to_vec()).unwrap()),
    //             flyer::ws::Event::Close(reason)   => println!("Websocket connection is close: {:?}", reason),
    //         }
    //     });
    // }, None);

    // print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    // server.listen();
}
// end: 'static