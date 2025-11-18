use flyer::{server, view::view_data};
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
    let mut server = server("127.0.0.1", 8888)
        .view("views");

    server.router().get("/", async |_req, res| {
        let mut data = view_data();

        data.insert("user", &User{
            first_name: "Jeo",
            last_name: "Deo"
        });

        return res.view("index.html", Some(data));
    });

    println!("Running Server: {}", server.address());

    server.listen();
}