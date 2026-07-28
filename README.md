# Flyer - Web Framework

## Information

Flyer web framework supports concurrent requests, allowing you to run requests without blocking each other.

### Supports

- HTTP/1.1
- HTTP/2.0
- HTTP/3.0
- WebSocket

## Getting Started

### Prerequisites

**Key features:**

- Router
- Subdomain
- View
- Env
- Assets
- Middleware
- Session
- Cookie
- Form and Multipart-Form
- Form Validation
- WebSocket
- Mail

### Getting with Flyer

First, create a new project:

```sh
cargo new example
cd example
```

Add `flyer` to your project:

```sh
cargo add flyer
```

---

## Examples

### 1. Basic Routing

This example demonstrates how to set up a basic HTTP server and define a simple GET route.

```rust
use flyer::server;

fn main() {
    let server = server("127.0.0.1", 9999);
    
    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```

### 2. Advanced Routing

Demonstrates route grouping, parameters, and HTTP methods.

```rust
use flyer::{server, request::Request, response::Response};

pub async fn index(_req: Request, res: Response) -> Response {
    return res.html("<h1>Users List</h1>");
}

pub async fn store(_req: Request, res: Response) -> Response {
    return res.redirect("users/1");
}

pub async fn view(req: Request, res: Response) -> Response {
    return res.html(format!("<h1>User {}</h1>", req.parameter("user")).as_str());
}

pub async fn update(req: Request, res: Response) -> Response {
    return res.redirect(format!("users/{}", req.parameter("user")).as_str());
}

pub async fn destroy(_req: Request, res: Response) -> Response {
    return res.redirect("users")
}

fn main() {
    let server = server("127.0.0.1", 9999);
    
    server.router().group("/", |router| {
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

### 3. Subdomain

To use subdomains locally, you must configure a local DNS resolver.

#### macOS/Linux
Create a resolver file:
```sh
sudo bash -c 'echo -e "nameserver 127.0.0.1 \nport 5354" > /etc/resolver/tracker.com'
```

#### Windows
You need to add a DNS client NRPT rule (requires PowerShell):
```powershell
Add-DnsClientNrptRule -Namespace "tracker.com" -NameServers "127.0.0.1" -Comment "Per-domain DNS for tracker"
```

#### Example Usage

```rust
use flyer::server;
use flyer::utils::development::dns;

fn main() {
    let server = server("127.0.0.1", 80);

    server.router().subdomain("api", |router| {
        router.get("/", async |_req, res| {
            return res.html("<h1>API Subdomain</h1>");
        });
    });

    server.init(async || {
        tokio::spawn(async {
            dns::run("tracker.com", "127.0.0.1", 5354);
        });
    });

    server.listen();
}
```

### 4. View Rendering

Flyer uses [Tera](https://keats.github.io/tera/) for view rendering.

Create `views/index.html`:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Hello {{ user.first_name }}</title>
</head>
<body>
    <h1>Hi, {{ user.first_name }} {{ user.last_name }}!</h1>
</body>
</html>
```

Main application:
```rust
use flyer::{server, view::{ViewData}};
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    first_name: &'static str,
    last_name: &'static str,
    email: &'static str
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .view("views");

    server.router().get("/", async |_req, res| {
        let mut data = ViewData::new();
        data.insert("user", &User{
            first_name: "Jeo",
            last_name: "Deo",
            email: "jeo.deo@gmail.com",
        });

        return res.view("index.html", Some(data));
    });

    server.listen();
}
```

### 5. Env

Create a `.env` file:
```env
HOST="127.0.0.1"
PORT="9999"
```

Application:
```rust
use flyer::{server, utils::env};

fn main() {
    env::load(".env");

    let host = env::env("HOST");
    let port: u32 = env::env("PORT").parse().unwrap_or(9999);

    let server = server(host, port)
        .view("views");
    
    server.listen();
}
```

### 6. Assets

Configure an assets directory to serve static files like CSS and JS.

`assets/style.css`:
```css
body { background-color: black; color: white; }
```

Application:
```rust
use std::time::Duration;
use flyer::{hooks::assets::AssetsHook, server, view::ViewData};

fn main() {
    let server = server("127.0.0.1", 9999)
        .hook(AssetsHook::new("assets", Duration::from_secs(3600), 1024 * 10));

    server.router().get("/", async |_req, res| {
        return res.view("index.html", Some(ViewData::new()));
    });

    server.listen();
}
```

### 7. Middleware

Use middleware for request interception and security.

```rust
use flyer::{
    request::Request,
    response::{HTTP_UNAUTHORIZED, Response},
    routing::next::Next,
    server
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonMessage { message: String }

pub async fn auth(req: Request, res: Response, next: Next) -> Response {
    if req.header("authorization") != "my-secret-token" {
        return res.status_code(HTTP_UNAUTHORIZED).json(&JsonMessage{
            message: "Unauthorized Access".to_owned()
        })
    }
    return next.handle(req, res);
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("api", |router| {
        router.get("/", async |_req, res| {
            return res.html("<h1>Authorized Access</h1>");
        });
    }).middleware(auth);

    server.listen();
}
```

### 8. Session

Manage user sessions securely.

```rust
use flyer::{request::Request, response::Response, server};

pub async fn home_view(req: Request, res: Response) -> Response {
    let user_id = req.session().get("user_id");
    return res.html(format!("<h1>Welcome user {}</h1>", user_id).as_str());
}

pub async fn login(_req: Request, res: Response) -> Response {
    return res.set_session("user_id", "1").back();
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", home_view);
        router.get("login", login);
    });

    server.listen();
}
```

### 9. Cookie

```rust
use std::time::Duration;
use flyer::{request::Request, response::Response, server};

pub async fn home_view(_req: Request, mut res: Response) -> Response {
    res.cookies().set("user_id", "1").set_expires(Duration::from_secs(3600));
    return res.html("<h1>Cookie set!</h1>");
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", home_view);

    server.listen();
}
```

### 10. Form & Multipart-Form

HTML for file upload:
```html
<form method="post" action="/upload" enctype="multipart/form-data">
    <input type="file" name="file">
    <button type="submit">Upload</button>
</form>
```

Application:
```rust
use flyer::{request::Request, response::Response, server, storage::local::LocalStorage};

pub async fn upload(req: Request, res: Response) -> Response {
    if let Some(file) = req.file("file") {
        let path = file.save("uploads").await.unwrap();
        return res.html("<h1>File uploaded!</h1>");
    }
    return res.html("<h1>No file uploaded!</h1>");
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .storage("default", LocalStorage::new("storage"));

    server.router().post("upload", upload);
    server.listen();
}
```

### 11. Form Validation

View for registration:
```html
<form action="/register" method="post">
    <input type="email" name="email">
    <input type="password" name="password">
    <button type="submit">Register</button>
</form>
```

Application:
```rust
use flyer::{
    request::Request, response::Response, routing::next::Next, server, validation::Rules,
};

pub async fn register_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();
    rules.rule("email", vec!["required", "email"]);
    rules.rule("password", vec!["required", "min:5"]);
    return rules.handle(req, res, next).await;
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().post("register", async |_req, res| {
        return res.html("<h1>Registration Successful!</h1>");
    }).middleware(register_form);

    server.listen();
}
```

### 12. WebSocket

```rust
use flyer::{server, websocket::{Websocket, WriterInterface}};

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().ws("/", async |_req, ws| -> Websocket {
        ws.on(async |event, writer| {
            if let flyer::websocket::Event::Text(bytes) = event {
                writer.write("Hello WebSocket!".into()).unwrap();
            }
        })
    });

    server.listen();
}
```

### 13. Mail

```rust
use flyer::{mail::Mail, server};
use uuid::Uuid;

fn main() {
    let server = server("127.0.0.1", 9999)
        .mailer("127.0.0.1".to_string(), 5555, "".to_string(), "".to_string(), false);
    
    server.router().get("/send-mail", async |_req, res| {
        Mail::new()
            .from("no-reply@test.com".to_string(), Some("no-reply"))
            .html(format!("<h1>Token: {}</h1>", Uuid::new_v4()))
            .send("user@test.com".to_string(), Some("User".to_string()))
            .unwrap();

        return res.html("<h1>Email sent!</h1>")
    });

    server.listen();
}
