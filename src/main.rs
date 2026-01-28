use flyer::server;

fn main() {
    let mut server = server("127.0.0.1", 9999);

    
    server.router().post("/", async |_req, res| {
        if let Some(file) = _req.file("file_0") {
            file.save_as("/", "image_0").await.unwrap();
        }        

        if let Some(file) = _req.file("file_1") {
            file.save("/").await.unwrap();
        }

        return res.html("<h1>Hello World!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}