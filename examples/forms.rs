use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    server, 
    session::cookie::new_session_manager,
    view::view_data
};

/*

TODO: Create file called index.html in views folder and paste html content below

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <base href="http://127.0.0.1:9999/">
  <title>Upload File</title>
  <style>
    body {
      text-align: center !important;
    }
  </style>
</head>
<body>
  <nav>
    <h1>Upload File</h1>
  </nav>
  <hr>
  <form method="post" action="/upload" enctype="multipart/form-data">
    <p style="color: red;">{{ error(name="file") }}</p>
    <p style="color: red;">{{ error_has(name="file") }}</p>
    <p style="color: red;">{{ error_has(name="file", class="is-invalid") }}</p>
    <input type="file" name="file" placeholder="Image">
    <br>
    <br>
    <br>
    <button type="submit">Upload File</button>
  </form>
</body>
</html>
```

*/

pub async fn home<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("index.html", Some(view_data()));
}

pub async fn upload<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    if req.file("file").is_none() {
        return res.with_error("file", "The file is required.")
            .back();
    }

    req.file("file").unwrap().save("file").await.unwrap();
    req.file("file").unwrap().save_as("storage", "file_backup").await.unwrap();

    return res.redirect("/");
}

fn main() {
    let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"))
        .view("views")
        .set_request_max_size(1024 * 100); // Max Request size 100MB

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}