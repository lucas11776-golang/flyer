# Flyer - Web Framework

## Getting Started

### Prerequisites

**Http key features:**

- Router         - 
- Response Types -
- Static Assets  -
- WebSocket      -
- Middleware     -
- Session        -


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
    
    server.router().group("api", |mut router| {
        router.group("users", |mut router| {
            router.get("/", index, None);
            router.post("/", store, None);
            router.group("{user}", |mut router| {
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
    }, Some(vec![auth]));

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Websocket

```rust
use flyer::server;

fn main() {
    let mut server = server("127.0.0.1", 9999);



    server.listen();
}
```