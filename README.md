# Flyer 🚀

Welcome to **Flyer**, a modern, high-performance, asynchronous web framework for Rust designed for building fast, scalable, and robust web applications and APIs. Flyer provides an expressive, batteries-included developer experience inspired by traditional high-level web frameworks while maintaining the raw speed, safety, and concurrency guarantees of Rust.

---

## 🌟 Core Information & Protocol Support

Flyer is built from the ground up on modern asynchronous primitives (powered by Tokio) to ensure that concurrent requests never block one another. 

### Supported Protocols
* **HTTP/1.1** & **HTTP/2.0** — Fully supported out-of-the-box for traditional web traffic and high-throughput API endpoints.
* **HTTP/3.0** — Next-generation transport layer support for minimal latency and improved connection migration.
* **WebSocket** — Real-time, bidirectional communication channels with robust event-handling hooks.

---

## 📦 Getting Started

### Feature Checklist
Flyer comes packed with modular features to handle everything from microservices to full-stack monoliths:
* 🔀 **High-Performance Router** with Grouping & Parameters
* 🌐 **Subdomain Routing** (with built-in local DNS utilities for development)
* 🎨 **View Engine** powered by Tera templates
* ⚙️ **Environment Configuration** management (`.env`)
* 📂 **Static Asset Management** with caching hooks
* 🛡️ **Middleware Interceptors** for security & logging
* 🍪 **Sessions & Cookies** state management
* 📤 **Multipart-Form Handling & File Uploads** (supporting local & cloud storage)
* ✅ **Form Validation Engine** with expressive rule sets
* 🔌 **WebSocket** asynchronous server events
* 📧 **Built-in Mailer** interface
* 🪝 **Custom Server Hooks** for request/response lifecycles
* 📋 **Custom Error Loggers** with built-in Sentry support

---

### Installation

Add `flyer` to your `Cargo.toml` dependency list via cargo:

```sh
cargo add flyer
```

---

## 💡 Examples & Guides

### 1. Basic Routing
Get a simple HTTP server running in just a few lines of code. The `server` helper binds your host and port, while closures handle routing asynchronously.

```rust
use flyer::server;

fn main() {
    // Initialize the server bound to 127.0.0.1 on port 9999
    let server = server("127.0.0.1", 9999);
    
    // Register a simple GET endpoint
    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World from Flyer!</h1>");
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    // Start listening for incoming connections
    server.listen();
}
```

---

### 2. Advanced Routing & Grouping
Flyer supports modular route groups, dynamic path parameters (via `{param}` syntax), and standard RESTful HTTP methods (`GET`, `POST`, `PATCH`, `DELETE`).

```rust
use flyer::{server, request::Request, response::Response};

pub async fn index(_req: Request, res: Response) -> Response {
    res.html("<h1>Users List</h1>")
}

pub async fn store(_req: Request, res: Response) -> Response {
    res.redirect("users/1")
}

pub async fn view(req: Request, res: Response) -> Response {
    let user_id = req.parameter("user");
    res.html(format!("<h1>User Profile: {}</h1>", user_id).as_str())
}

pub async fn update(req: Request, res: Response) -> Response {
    let user_id = req.parameter("user");
    res.redirect(format!("users/{}", user_id).as_str())
}

pub async fn destroy(_req: Request, res: Response) -> Response {
    res.redirect("users")
}

fn main() {
    let server = server("127.0.0.1", 9999);
    
    // Group routes under prefixes cleanly
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

---

### 3. Subdomain Routing
Flyer makes multi-tenant or modular subdomain architectures straightforward. For local development, Flyer includes built-in DNS utilities.

#### Local DNS Resolver Setup
* **macOS / Linux:**
  ```sh
  sudo bash -c 'echo -e "nameserver 127.0.0.1 \nport 5354" > /etc/resolver/tracker.com'
  ```
* **Windows (PowerShell):**
  ```powershell
  Add-DnsClientNrptRule -Namespace "tracker.com" -NameServers "127.0.0.1" -Comment "Per-domain DNS for tracker"
  ```

#### Example Usage
```rust
use flyer::server;
use flyer::utils::development::dns;

fn main() {
    let server = server("127.0.0.1", 80);

    // Bind routes exclusively to the "api" subdomain
    server.router().subdomain("api", |router| {
        router.get("/", async |_req, res| {
            res.html("<h1>Welcome to the API Subdomain</h1>")
        });
    });

    // Spin up the local development DNS resolver asynchronously alongside the server
    server.init(async || {
        tokio::spawn(async {
            dns::run("tracker.com", "127.0.0.1", 5354);
        });
    });

    server.listen();
}
```

---

### 4. View Rendering (Tera Integration)
Flyer integrates directly with the [Tera](https://keats.github.io/tera/) templating engine for powerful server-side rendering.

**Template File (`views/index.html`):**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Welcome, {{ user.first_name }}</title>
</head>
<body>
    <h1>Hi, {{ user.first_name }} {{ user.last_name }}!</h1>
    <p>Contact: {{ user.email }}</p>
</body>
</html>
```

**Rust Application:**
```rust
use flyer::{server, view::ViewData};
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    first_name: &'static str,
    last_name: &'static str,
    email: &'static str,
}

fn main() {
    // Register the directory containing templates
    let server = server("127.0.0.1", 9999)
        .view("views");

    server.router().get("/", async |_req, res| {
        let mut data = ViewData::new();
        data.insert("user", &User {
            first_name: "Jane",
            last_name: "Doe",
            email: "jane.doe@gmail.com",
        });

        res.view("index.html", Some(data))
    });

    server.listen();
}
```

---

### 5. Environment Configuration
Manage configuration parameters safely using standard `.env` files.

**`.env` file:**
```env
HOST="127.0.0.1"
PORT="9999"
```

**Rust Application:**
```rust
use flyer::{server, utils::env};

fn main() {
    env::load(".env");

    let host = env::env("HOST");
    let port: u32 = env::env("PORT").parse().unwrap_or(9999);

    let server = server(host, port).view("views");
    
    server.listen();
}
```

---

### 6. Static Asset Hook
Serve static files like CSS, JavaScript, and images efficiently with built-in caching control.

**`assets/style.css`:**
```css
body {
    background-color: #121212;
    color: #e0e0e0;
    font-family: sans-serif;
}
```

**Rust Application:**
```rust
use std::time::Duration;
use flyer::{hooks::assets::AssetsHook, server, view::ViewData};

fn main() {
    // Attach an asset hook mapping the local "assets" folder with a 1-hour cache duration
    let server = server("127.0.0.1", 9999)
        .hook(AssetsHook::new("assets", Duration::from_secs(3600), 1024 * 10));

    server.router().get("/", async |_req, res| {
        res.view("index.html", Some(ViewData::new()))
    });

    server.listen();
}
```

---

### 7. Middleware Interceptors
Middleware allows you to inspect, filter, or modify requests and responses globally or per route group (e.g., for authentication).

```rust
use flyer::{
    request::Request,
    response::{HTTP_UNAUTHORIZED, Response},
    routing::next::Next,
    server
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonMessage { 
    message: String 
}

pub async fn auth(req: Request, res: Response, next: Next) -> Response {
    if req.header("authorization") != "my-secret-token" {
        return res.status_code(HTTP_UNAUTHORIZED).json(&JsonMessage {
            message: "Unauthorized Access: Invalid Token".to_owned()
        });
    }
    // Pass control to the next middleware or final handler
    next.handle(req, res).await
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("api", |router| {
        router.get("/", async |_req, res| {
            res.html("<h1>Authorized Access Granted</h1>")
        });
    }).middleware(auth);

    server.listen();
}
```

---

### 8. Session Management
Maintain persistent user state across HTTP requests using Flyer's session store.

```rust
use flyer::{request::Request, response::Response, server};

pub async fn home_view(req: Request, res: Response) -> Response {
    let user_id = req.session("user_id").unwrap_or_default();
    res.html(format!("<h1>Welcome back, user #{}</h1>", user_id).as_str())
}

pub async fn login(_req: Request, res: Response) -> Response {
    // Set a session variable and redirect back
    res.set_session("user_id", "42").back()
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

---

### 9. Cookies
Easily set, inspect, and configure secure HTTP cookies.

```rust
use std::time::Duration;
use flyer::{request::Request, response::Response, server};

pub async fn home_view(_req: Request, mut res: Response) -> Response {
    res.cookies()
        .set("user_id", "42")
        .set_expires(Duration::from_secs(3600));
        
    res.html("<h1>Cookie has been successfully set!</h1>")
}

fn main() {
    let server = server("127.0.0.1", 9999);
    server.router().get("/", home_view);
    server.listen();
}
```

---

### 10. Forms & Multipart File Uploads
Handle multipart form data securely and save uploaded files directly using local or abstract storage layers.

**Rust Application:**
```rust
use flyer::{
    request::Request,
    response::Response,
    server,
    storage::local::LocalStorage,
};

pub async fn home(_req: Request, res: Response) -> Response {
    return res.html(r#"
        <form method="post" action="/upload" enctype="multipart/form-data">
            <h1>Upload File/Files</h1>
            <input type="file" name="file" multiple>
            <button type="submit">Upload</button>
        </form>
    "#);
}

pub async fn upload(req: Request, res: Response) -> Response {
    if req.files().len() > 0 {
        for (_, file) in req.files() {
            file
                .save_as("", &file.name)
                .await
                .unwrap();
        }
        return res.html("<h1>File uploaded!</h1>");
    }
    return res.html("<h1>No file uploaded!</h1>");
}

fn main() {
    let server = server("127.0.0.1", 9999)
        .storage("default", LocalStorage::new("storage"));

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```

---

### 11. Form Validation
Validate incoming request payloads declaratively with custom validation rules.

**HTML Form:**
```html
<form action="/register" method="post">
    <input type="email" name="email" placeholder="Email">
    <input type="password" name="password" placeholder="Password">
    <button type="submit">Register</button>
</form>
```

**Rust Application:**
```rust
use flyer::{
    request::Request, response::Response, routing::next::Next, server, validation::Rules,
};

pub async fn register_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();
    rules.rule("email", vec!["required", "email"]);
    rules.rule("password", vec!["required", "min:5"]);
    
    rules.handle(req, res, next).await
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().post("register", async |_req, res| {
        res.html("<h1>Registration Successful!</h1>")
    }).middleware(register_form);

    server.listen();
}
```

---

### 12. WebSockets
Build high-performance, real-time bidirectional communication channels.

```rust
use flyer::{server, websocket::{Websocket, WriterInterface}};

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().ws("/", async |_req, ws| -> Websocket {
        ws.on(async |event, writer| {
            if let flyer::websocket::Event::Text(text_data) = event {
                println!("Received message: {}", text_data);
                let _ = writer.write("Hello from Flyer WebSocket Server!".into());
            }
        })
    });

    server.listen();
}
```

---

### 13. Mailer Integration
Send transactional emails out of the box using built-in mail utilities and template strings.

```rust
use flyer::{mail::Mail, server};
use uuid::Uuid;

fn main() {
    // Configure mailer parameters (host, port, username, password, tls)
    let server = server("127.0.0.1", 9999)
        .mailer("127.0.0.1".to_string(), 5555, "".to_string(), "".to_string(), false);
    
    server.router().get("/send-mail", async |_req, res| {
        let _ = Mail::new()
            .from("no-reply@test.com".to_string(), Some("No-Reply"))
            .html(format!("<h1>Your Verification Token: {}</h1>", Uuid::new_v4()))
            .send("user@test.com".to_string(), Some("User Name".to_string()));

        res.html("<h1>Email sent successfully!</h1>")
    });

    server.listen();
}
```

---

### 14. Custom Server Hooks
Custom server hooks allow you to intercept request/response cycles before or after your routes and middleware are executed.

```rust
use flyer::{
    hooks::Hook,
    request::Request,
    response::Response,
    routing::next::Next,
    server
};

pub struct CustomHook { }

impl CustomHook {
    pub fn new() -> Self {
        Self { }
    }
}

impl Hook for CustomHook {
    async fn before(&self, req: Request, res: Response, next: Next) ->  Response {
        // Do some work before request hits middleware and route.
        println!("BEFORE HOOK");
        next.handle(req, res)
    }

    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        // Do some work after request hits middleware and route.
        println!("AFTER HOOK");
        next.handle(req, res)
    }
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", async |_req, res| {
        println!("CONTROLLER");
        res.html("<h1>Hello controller</h1>")
    });

    server.hook(CustomHook::new());

    server.listen();
}
```

---

### 15. Custom Error Loggers
Flyer allows you to define custom error logging behavior for your application, including built-in support for Sentry.

```rust
use flyer::{
    error::logger::{Logger, PanicErrorInfo},
    loggers::sentry::Sentry,
    request::Request,
    response::Response, server
};

pub struct DebuggerLogger { }

impl DebuggerLogger {
    pub fn new() -> Self {
        Self { }
    }
}

impl Logger for DebuggerLogger {
    async fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> () {
        println!("\r\n\r\nError: {}\r\nMessage: {}\r\nPath: {}r\n\r\n", info.error, info.message, req.path());
    }
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            res.html("<h1>Page With No Error Show</h1>")
        });
        router.get("error", async |_req, res| {
            let user_id: Option<String> = None;

            user_id.unwrap();

            res.html("<h1>Page With Error Will No Show</h1>")
        });
    });

    // Custom logger
    server.logger(DebuggerLogger::new());

    // Prebuilt logger for sentry
    // server.logger(Sentry::new("SENTRY_DNS", "ENVIRONMENT"));

    server.listen();
}
```

---

## 🎨 Tera View Template Built-in Functions

Flyer exposes a rich set of helper functions ready to be used directly inside your Tera templates for sessions, validation feedback, and environment variables.

### Session Functions
| Function      | Description                                            | Usage Example                   |
| :------------ | :----------------------------------------------------- | :------------------------------ |
| `session`     | Retrieves a value from the active session by key.      | `{{ session(name="key") }}`     |
| `session_has` | Checks if a key currently exists in the session store. | `{{ session_has(name="key") }}` |

### Validation & Flash Feedback Functions
| Function    | Description                                                               | Usage Example                 |
| :---------- | :------------------------------------------------------------------------ | :---------------------------- |
| `error`     | Retrieves a validation error string for a specific form field.            | `{{ error(name="key") }}`     |
| `error_has` | Checks whether a validation error exists for a given field.               | `{{ error_has(name="key") }}` |
| `old`       | Retrieves previously submitted input values following validation failure. | `{{ old(name="key") }}`       |
| `flash`     | Retrieves a temporary session flash message.                              | `{{ flash(name="key") }}`     |
| `flash_has` | Checks if a flash message is present.                                     | `{{ flash_has(name="key") }}` |

### Utility Functions
| Function | Description                                                           | Usage Example                 |
| :------- | :-------------------------------------------------------------------- | :---------------------------- |
| `env`    | Retrieves an environment variable directly in the template.           | `{{ env(name="KEY") }}`       |
| `url`    | Automatically generates a full URL path for named or standard routes. | `{{ url(path="/my-route") }}` |
