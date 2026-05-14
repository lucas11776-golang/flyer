use flyer::{server, view::{ViewData, render_view}};
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

/*

TODO: Create file called render.html in views folder and paste html content below

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>This view is render using render_view</title>
</head>
<body>
    <h1>Hello, {{ user.first_name }} welcome to the community</h1>
</body>
</html>
```

*/

#[derive(Serialize)]
pub struct User<'a> {
    first_name: &'a str,
    last_name: &'a str,
    email: &'a str
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .view("views");


    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            let mut data = ViewData::new();

            data.insert("user", &User{
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo.deo@gmail.com",
            });

            return res.view("index.html", Some(data));
        });
        router.get("/render", async |_req, res| {
            let user = User{
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo.deo@gmail.com",
            };

            // This helper function is useful when sending email`s etc.
            let html = render_view("render.html", Some(ViewData::with("user", &user))).unwrap();

            return res.html(&html);
        });
    });

    println!("Running Server: {}", server.address());

    server.listen();
}