
use flyer::{server, view::view_data};

fn main() {
    let mut server = server("127.0.0.1", 9999);

    // Create view folder in base project directory.
    server.view("views");
    
    server.router().get("/", async |req, res| {
        let mut data = view_data();

        data.insert("first_name", "Jeo");
        data.insert("last_name", "Doe");
        data.insert("email", "jeo@doe.com");
        data.insert("age", &23);

        // Create file called index.html in views folder.
        return res.view("index.html", Some(data))
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}