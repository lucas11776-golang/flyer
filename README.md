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

pub async fn index(_req: Request, res: Response) -> Response {
    return res
}

pub async fn store(_req: Request, res: Response) -> Response {
    return res
}

pub async fn view(_req: Request, res: Response) -> Response {
    return res
}

pub async fn update(_req: Request, res: Response) -> Response {
    return res
}

pub async fn destroy(_req: Request, res: Response) -> Response {
    return res
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
    server.router().group("api", |mut router| {
        router.group("users", |mut router| {
            router.get("/", index, None);
            router.post("/", store, None);
            router.group("{user}", |mut router| {
                router.patch("/", update, None);
                router.delete("/", destroy, None);
            }, None);
        }, None);
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```