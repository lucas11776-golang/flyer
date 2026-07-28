use flyer::{
    request::Request,
    response::Response,
    server,
    storage::local::LocalStorage,
};

/// File Upload & Storage Example
///
/// This example demonstrates:
/// - Handling multipart/form-data
/// - Accessing files from a request
/// - Saving files to storage
/// - Basic storage management
pub async fn home(_req: Request, res: Response) -> Response {
    return res.html(r#"
        <form method="post" action="/upload" enctype="multipart/form-data">
            <input type="file" name="file">
            <button type="submit">Upload</button>
        </form>
    "#);
}

pub async fn upload(req: Request, res: Response) -> Response {
    if let Some(file) = req.file("file") {
        // Save file using default storage
        let path = file.save("uploads").await.unwrap();
        println!("File saved to: {}", path);
        return res.html("<h1>File uploaded!</h1>");
    }

    return res.html("<h1>No file uploaded!</h1>");
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .storage("default", LocalStorage::new("storage"));

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
