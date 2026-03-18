use flyer::{server, view::ViewData};
use serde::Serialize;

/*

TODO: Create file called index.html in views folder and paste html content below

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello {{ user.first_name }}</title>
</head>
<body>
    <h1>Hi, {{ user.first_name }} {{ user.last_name }} how are you?</h1>
</body>
</html>
```

*/

#[derive(Serialize)]
pub struct User<'a> {
    first_name: &'a str,
    last_name: &'a str,
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .view("views");

    server.router().get("/", async |_req, res| {
        let mut data = ViewData::new();

        data.insert("user", &User{
            first_name: "Jeo",
            last_name: "Deo"
        });

        return res.view("index.html", Some(data));
    });

    println!("Running Server: {}", server.address());

    server.listen();
}