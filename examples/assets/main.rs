use flyer::{server, view::view_data};

/*

TODO: Create file called `style.css` in folder called `assets` and copy the content below in the file.

```css
.body {
    background-color: block;
}

h1 {
    color: white;
}
```

TODO: Create file called `index.html` in folder called `views` and copy the content below in the file.

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <base href="http://127.0.0.1:9999/">
  <title>Assets Test</title>
  <link href="/style.css" rel="stylesheet">
</head>
<body>
  <h1>Hello World</h1>
</body>
</html>
```

*/

fn main() {
    let mut server = server("127.0.0.1", 8888)
        .view("views");

    server.router().get("/", async |_req, res| {
        return res.view("index.html", Some(view_data()));
    });

    println!("Running Server: {}", server.address());

    server.listen();
}