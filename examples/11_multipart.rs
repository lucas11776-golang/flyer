use flyer::{
    request::Request,
    response::Response,
    server,
    storage::local::LocalStorage,
};

pub async fn home(_req: Request, res: Response) -> Response {
    return res.html(r#"
        <form method="post" action="/upload" enctype="multipart/form-data">
            <h1>Upload File/Files</h1>
            <input type="file" name="file" multiple>
            <button type="submit">Upload</button>
        </form>
    "#);
}

pub async fn upload(req: Request, res: Response) -> Response {
    if req.files().len() > 0 {
        for (_, file) in req.files() {
            file
                .save_as("", &file.name)
                .await
                .unwrap();
        }
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
