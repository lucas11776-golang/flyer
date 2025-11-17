# Flyer - Web Framework

## Getting Started

### Prerequisites

**Http key features:**

- Router
- View
- Assets
- Middleware
- Session
- Cookie
- WebSocket


## Getting with Flyer

First create a new project using command:

```sh
cargo new example
```

After running the command add `flyer` to you project using command:

```sh
cargo add flyer
```

### Running HTTP server

In order to run a basic server `copy` and `paste` below `code snippet`.

```rs
use flyer::server;

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```

Now we are ready to run the server using command.

```sh
cargo run
```

if you want to run secure server you can use function `server_tls` here is example below.

```rs
use flyer::server_tls;

fn main() {
    let mut server = server_tls("127.0.0.1", 9999, ":HOST_KEY_PATH:", ":HOST_CERT_PATH:");
    
    server.router().get("/", async |req, res| {
        return res.html("<h1>Hello World Secure Connection!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Router

```rs
use flyer::{server, request::Request, response::Response};

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn store<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn view<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn update<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

pub async fn destroy<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().group("api", |router| {
        router.group("users", |router| {
            router.get("/", index);
            router.post("/", store);
            router.group("{user}", |router| {
                router.get("/", view);
                router.patch("/", update);
                router.delete("/", destroy);
            });
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### View

Create file called `index.html` in folder called views and copy the content below in the file.

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

The next step to insert code below in `main.rs`.

```rust
use flyer::{server, view::view_data};
use serde::Serialize;

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
```

For more information about view functionality view [Tera](https://keats.github.io/tera/).


### Assets

Create file called `style.css` in folder called `assets` and copy the content below in the file.

```css
body {
    background-color: black;
}

h1 {
    color: white;
}
```

And also create file called `index.html` in folder called `views` and copy the content below in the file.

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

The next step to insert code below in `main.rs`.

```rs
use flyer::{server, view::view_data};

fn main() {
    let mut server = server("127.0.0.1", 8888)
        .view("views");

    server.router().get("/", async |_req, res| {
        return res.view("index.html", Some(view_data()));
    });

    println!("Running Server: {}", server.address());

    server.listen();
}
```

You should see background color of black and Hello World with white color if you visit `127.0.0.0.1:9999`


### Middleware

```rust
use flyer::{server, request::Request, response::Response, router::Next};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: u64,
    email:  String
}

#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
    message: String
}

pub async fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    if req.header("authorization") != "ey.jwt.token" {
        return res.status_code(401).json(&JsonMessage{
            message: "Unauthorized Access".to_owned()
        })
    }
    
    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().get("api/users/{user}", async |req, res| {
        return res.json(&User{
            id: req.parameter("user").parse().unwrap(),
            email: "joe@deo.com".to_owned()
        })
    }).middleware(auth);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Session

```rust
use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    router::Next,
    server,
    session::cookie::new_session_manager
};

/// Controller
pub async fn home_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html(format!("<h1>Welcome to protected home page user {}</h1>", req.session().get("user_id")).as_str());
}

pub async fn login<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.session().set("user_id", format!("{}", 1).as_str());

    return res.redirect("login");
}

pub async fn register<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html("<h1>Please visit the login page to login</h1>");
}

pub async fn logout<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.session().remove("user_id");

    return res.redirect("register");
}

pub async fn page_not_found<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html("<h1>404 Page Not Found</h1>");
}

/// Middleware
pub async fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.session().get("user_id") == "" {
        return res.redirect("register");
    }

    return next.handle(res);
}

pub async fn guest<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.session().get("user_id") != "" {
        return res.redirect("/");
    }

    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"));

    server.router().group("/", |router| {
        router.get("/", home_view)
            .middleware(auth);
        router.get("register", register)
            .middleware(guest);
        router.get("login", login)
            .middleware(guest);
        router.get("logout", logout)
            .middleware(auth);
    });

    server.router().not_found(page_not_found);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Cookie

```rs
use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    server,
};

pub async fn home_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.cookies()
        .set("user_id", "1")
        .set_expires(Duration::from_hours(2));

    return res.html("<h1>Cookie has been set visit route /cookie</h1>");
}

pub async fn cookie<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html(format!("<h1>User ID cookie is {}</h1>", req.cookies().get("user_id")).as_str());
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", home_view);
        router.get("cookie", cookie);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Websocket

```rust
use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    server,
};

pub async fn home_view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    req.cookies()
        .set("user_id", "1")
        .set_expires(Duration::from_hours(2));

    return res.html("<h1>Cookie has been set visit route /cookie</h1>");
}

pub async fn cookie<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html(format!("<h1>User ID cookie is {}</h1>", req.cookies().get("user_id")).as_str());
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", home_view);
        router.get("cookie", cookie);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```