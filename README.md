# Flyer - Web Framework

## Getting Started

### Prerequisites

**Http key features:**

- Router
- Response
- Static Assets
- WebSocket
- Middleware
- Session


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
    
    server.router().get("/", async |req, res| {
        return res.html("<h1>Hello World!!!</h1>")
    }, None);

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
    }, None);

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
            router.get("/", index, None);
            router.post("/", store, None);
            router.group("{user}", |router| {
                router.get("/", view, None);
                router.patch("/", update, None);
                router.delete("/", destroy, None);
            }, None);
        }, None);
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


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

// TODO: working on async middleware
pub fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
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
    }, Some(vec![auth]));

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### View

If create file called `index.html` in folder called views and copy the content below in the file.

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
    }, None);

    println!("Running Server: {}", server.address());

    server.listen();
}
```

For more information about view functionality view [Tera](https://keats.github.io/tera/).


### Websocket

```rust
use flyer::{server};
use flyer::{request::Request, response::Response, router::Next};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<'a> {
    message: &'a str
}

pub fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    if req.header("authorization") != "jwt.token" {
        let writer = res.ws.as_mut().unwrap();

        writer.write(serde_json::to_vec(&Message{message: "Unauthorized Access"}).unwrap());
        
        return res;
    }

    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("", |router| {
        router.ws("/", async |_req, ws| {
            ws.on(async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(_items) => writer.write("Hello This Public Route".into()),
                    flyer::ws::Event::Binary(_items) => todo!(),
                    flyer::ws::Event::Ping(_items) => todo!(),
                    flyer::ws::Event::Pong(_items) => todo!(),
                    flyer::ws::Event::Close(_reason) => todo!(),
                }
            });
        }, None);

        router.ws("/private", async |_req, ws| {
            ws.on(async |event, writer| {
                match event {
                    flyer::ws::Event::Ready() => todo!(),
                    flyer::ws::Event::Text(_items) => writer.write("Hello This Private Route".into()),
                    flyer::ws::Event::Binary(_items) => todo!(),
                    flyer::ws::Event::Ping(_items) => todo!(),
                    flyer::ws::Event::Pong(_items) => todo!(),
                    flyer::ws::Event::Close(_reason) => todo!(),
                }
            });
        },Some(vec![auth]));
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```