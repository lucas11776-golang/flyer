use flyer::{server_tls, view::view_data};

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

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}