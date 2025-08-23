# Flyer - Web Framework

## Getting Started

### Prerequisites

**Http key features:**

- Router         - 
- Response Types -
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
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = flyer::server("127.0.0.1", 9999).await?;

    server.router().get("/", |_req, res| {
        return res.html("<h1>Hello World!!!</h1>");
    }, None);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen().await;

    Ok(())
}
```

Now we are ready to run the server using command.

```sh
cargo run
```