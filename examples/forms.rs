use std::time::Duration;

use flyer::{
    request::Request,
    response::{HTTP_OK, Response},
    router::next::Next,
    server,
    session::cookie::SessionCookieManager,
    storage::{self, DEFAULT_STORAGE, local::LocalStorage},
    validation::Rules,
    view::ViewData
};
use serde_json::json;

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
    return res.view("index.html", Some(ViewData::new()));
}

#[allow(unused)]
pub async fn upload<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    if req.file("file").is_none() {
        return res.with_error("file", "The file is required.")
            .back();
    }

    // Save from `File` use default storage
    let req_save_0 = req.file("file").unwrap().save("file").await.unwrap();
    let req_backup_0 = req.file("file").unwrap().save_as("backup", "backup").await.unwrap();

    println!("FROM REQUEST SAVE PATH {}", req_save_0);
    println!("FROM REQUEST SAVE_AS {}", req_backup_0);

    if let Ok(exists) = storage::exists(DEFAULT_STORAGE, &req_save_0) && exists {
        println!("File exists: {}", req_save_0);
    }

    if let Ok(_) = storage::delete(DEFAULT_STORAGE, &req_backup_0) {
        println!("File deleted {}", req_save_0);
    }

    // Storage helper functions
    let req_save_1 = storage::save(DEFAULT_STORAGE, "file", req.file("file").unwrap()).unwrap();
    let req_backup_1 = storage::save_as(DEFAULT_STORAGE, "backup", "backup_1", req.file("file").unwrap()).unwrap();
    let exists = storage::exists(DEFAULT_STORAGE, &req_save_1).unwrap();
    let file = storage::get(DEFAULT_STORAGE, &req_save_1).unwrap();
    storage::delete(DEFAULT_STORAGE, &req_save_1).unwrap();

    return res.redirect("/");
}

pub async fn json_form<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    let mut rules = Rules::new();

    rules.rule("first_name", vec!["required", "string", "min:3", "max:50"]);
    rules.rule("last_name", vec!["required", "string", "min:3", "max:50"]);
    rules.rule("email", vec!["required", "email"]);

    return rules.handle(req, res, next);
}

pub async fn json<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.status_code(HTTP_OK).json(&json!({"message": "Entity created"}));
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .session(SessionCookieManager::new(Duration::from_secs((60 * 60) * 2), "session_cookie_key_name", "encryption"))
        .storage(DEFAULT_STORAGE, LocalStorage::new(Some("storage")))
        .view("views")
        .set_request_max_size(1024 * 100); // Max Request size 100MB

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
        router.post("json", json).middleware(json_form);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}