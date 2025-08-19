# HTTP

## Getting Started

### Prerequisites

**Http key features:**

- Router         - 
- Response Types -
- Static Assets
- WebSocket
- Middleware
- Session


## Getting with HTTP

### Running HTTP server

Create a file called `main.rs` in you project and paste the below code.

```rs
use std::io::Result;

mod http;

fn main() -> Result<()> {
    let mut serve = http::server("127.0.0.1".to_string(), 9999)?;

    serve.router().get("/".to_owned(), |req, res| {
        return res.html("<h1>Hello World!!!</h1>".to_owned());
    });

    serve.listen();

    Ok(())
}
```