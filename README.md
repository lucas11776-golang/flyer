# Flyer 🚀

Welcome to **Flyer**, a modern, high-performance, asynchronous web framework for Rust designed for building fast, scalable, and robust web applications and APIs. Flyer provides an expressive, batteries-included developer experience inspired by traditional high-level web frameworks while maintaining the raw speed, safety, and concurrency guarantees of Rust.

---

## 🤖 Guide for AI Agents & Automated Developers

When building applications or generating code with Flyer, adhere to the following architectural conventions:
- **Server Initialization**: Always use `flyer::server("HOST", PORT)` to instantiate the server.
- **Asynchronous Handlers**: Route handlers are asynchronous closures or functions taking `Request` and `Response`, returning `Response` (e.g., `async |_req, res| { res.html("...") }`).
- **Routing & Groups**: Use `server.router().group("prefix", |router| { ... })` for nested route grouping and parameter extraction (`req.parameter("name")`).
- **View Rendering**: Tera templates are supported via `.view("views_dir")` and `res.view("template.html", Some(data))`. Dioxus UI components are supported via `res.component(Component)` or `res.component_with_props(Component, props)`.
- **Validation**: Declarative validation uses `validation::Rules` inside middleware (`rules.handle(req, res, next).await`).
- **Session & Cookies**: Managed directly through `req.session(...)`, `res.set_session(...)`, `res.cookies()`, and `req.cookies()`.

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
* 🔀 **High-Performance Router** with Grouping & Parameters (`GET`, `POST`, `PATCH`, `DELETE`, `WS`)
* 🌐 **Subdomain Routing** (with built-in local DNS utilities for development)
* 🎨 **View Engine** powered by Tera templates
* ⚛️ **UI Component Rendering** powered by Dioxus
* ⚙️ **Environment Configuration** management (`.env`)
* 📂 **Static Asset Management** with caching hooks
* 🛡️ **Middleware Interceptors** for security & logging
* 🍪 **Sessions & Cookies** state management
* 📤 **Multipart-Form Handling & File Uploads** (local storage and abstract storage layers)
* ✅ **Form Validation Engine** with expressive rule sets
* 🔌 **WebSocket** asynchronous server events
* 📧 **Built-in Mailer** interface
* 🪝 **Custom Server Hooks** for request/response lifecycles
* 📋 **Custom Error Loggers** with built-in Sentry support

### Installation

Add `flyer` to your `Cargo.toml` dependency list via cargo:

```sh
cargo add flyer
```

---

## 💡 Examples & Guides (All 17 Modules)

### 1. Hello World (`examples/01_hello_world.rs`)
Get a simple HTTP server running in just a few lines of code.

```rust
use flyer::server;

pub fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", async |_req, res| {
        res.html("<h1>Hello World</h1>")
    });

    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());

    server.listen();
}
```

---

### 2. Controller Functions (`examples/02_controller.rs`)
Organize your route logic into modular controller functions.

```rust
use flyer::{request::Request, response::Response, server};

pub async fn index(_req: Request, res: Response) -> Response {
    res.html("<h1>Controller</h1>")
}

pub fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", index);

    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());

    server.listen();
}
```

---

### 3. Basic Routing (`examples/03_routing.rs`)
Handle grouped routes and custom 404 not found pages.

```rust
use flyer::{
    request::Request,
    response::{HTTP_NOT_FOUND, Response},
    server
};

pub async fn index(_req: Request, res: Response) -> Response {
    res.html("<h1>Hello World!!!</h1>")
}

pub async fn not_found(_req: Request, res: Response) -> Response {
    res
        .status_code(HTTP_NOT_FOUND)
        .html("<h1>Hello World!!!</h1>")
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            return res.html("<h1>Hello World!!!</h1>")
        });
    });

    server.router().not_found(not_found);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```

---

### 4. Advanced Routing & Parameters (`examples/04_advanced_routing.rs`)
Flyer supports nested route groups, dynamic path parameters (via `{user}` syntax), and full error handling handlers.

```rust
use flyer::{
    error::Error,
    request::Request,
    response::{HTTP_NOT_FOUND, Response},
    routing::next::Next,
    server
};

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
    let get_user_id: Option<String> = None;
    get_user_id.unwrap(); // Triggers error handler
    return res.redirect("users")
}

pub async fn not_found(_req: Request, res: Response) -> Response {
    res
        .status_code(HTTP_NOT_FOUND)
        .html("<h1>Hello World!!!</h1>")
}

pub async fn error(_error: Error, _req: Request, res: Response, _next: Next) -> Response {
    res
        .status_code(HTTP_NOT_FOUND)
        .html("<h1>500 Internal Server Error</h1>")
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

    server.router().not_found(not_found);
    server.error(error);

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 5. View Rendering (`examples/05_views.rs`)
Render HTML templates using Tera templates with strongly typed data structures.

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

    println!("Running Server: {}", server.address());
    server.listen();
}
```

---

### 6. Component Rendering / Dioxus UI (`examples/06_component.rs`)
Render reactive UI components using Dioxus directly from route handlers.

```rust
use flyer::server;
use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct AccountPageProps {
    first_name: &'static str,
    last_name: &'static str,
    email: &'static str
}

#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            h1 { "Welcome Home Page" }
        }
    }
}

#[allow(non_snake_case)]
pub fn AccountPage(account: AccountPageProps) -> Element {
    rsx! {
        div {
            h1 { "Fullname: {account.first_name} {account.last_name}" }
            h1 { "Email: {account.email}" }
        }
    }
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            res.component(HomePage)
        });
        router.get("/account", async |_req, res| {
            res.component_with_props(AccountPage, AccountPageProps{
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo.deo@gmail.com",
            })
        });
    });

    println!("\r\n\r\nRunning Server: {}", server.address());
    server.listen();
}
```

---

### 7. Environment Configuration (`examples/07_env.rs`)
Load `.env` configuration variables safely.

```rust
use flyer::{
    request::Request,
    response::Response,
    server,
    utils::{env}
};

pub async fn index(_req: Request, res: Response) -> Response {
    return res.html("<h1>Check console for environment details</h1>");
}

fn main() {
    env::load(".env");

    let host = env::env("HOST");
    let port: u32 = env::env("PORT").parse().unwrap_or(9999);

    let server = server(host, port).view("views");

    server.router().group("/", |router| {
        router.get("/", index);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 8. Static Assets (`examples/08_assets.rs`)
Serve static assets with caching headers.

```rust
use std::time::Duration;
use flyer::{hooks::assets::AssetsHook, server, view::ViewData};

fn main() {
    let server = server("127.0.0.1", 9999)
        .hook(AssetsHook::new("assets", Duration::from_secs(60 * 60), 1024 * 10))
        .view("views");

    server.router().get("/", async |_req, res| {
        return res.view("index.html", Some(ViewData::new()));
    });

    println!("Running Server: {}", server.address());
    server.listen();
}
```

---

### 9. Middleware Interceptors (`examples/09_middleware.rs`)
Secure route groups using custom middleware.

```rust
use flyer::{
    request::Request,
    response::{HTTP_UNAUTHORIZED, Response},
    routing::next::Next, server
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
    message: String
}

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

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 10. Session Management (`examples/10_session.rs`)
Manage server-side user sessions.

```rust
use flyer::{request::Request, response::Response, server};

pub async fn home_view(req: Request, res: Response) -> Response {
    let user_id = req.session("user_id");
    if user_id == "" {
        return res.html("<h1>Not logged in</h1>");
    }
    return res.html(format!("<h1>Welcome user {}</h1>", user_id).as_str());
}

pub async fn login(_req: Request, res: Response) -> Response {
    return res.set_session("user_id", "1").back();
}

pub async fn logout(_req: Request, res: Response) -> Response {
    return res.remove_session("user_id").back();
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", home_view);
        router.get("login", login);
        router.get("logout", logout);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 11. Cookies (`examples/11_cookie.rs`)
Set, retrieve, and expire HTTP cookies.

```rust
use std::time::Duration;
use flyer::{request::Request, response::Response, server};

pub async fn home_view(_req: Request, mut res: Response) -> Response {
    res.cookies()
        .set("user_id", "1")
        .set_expires(Duration::from_secs(3600));

    return res.html("<h1>Cookie has been set! Visit <a href='/cookie'>/cookie</a></h1>");
}

pub async fn cookie(req: Request, res: Response) -> Response {
    let cookie_val = req.cookies().get("user_id");
    return res.html(format!("<h1>User ID cookie is {:?}</h1>", cookie_val).as_str());
}

pub async fn remove_cookie(_req: Request, mut res: Response) -> Response {
    res.cookies().remove("user_id");
    return res.redirect("/");
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", home_view);
        router.get("cookie", cookie);
        router.delete("cookie/remove", remove_cookie);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 12. Multipart File Uploads (`examples/12_multipart.rs`)
Handle file uploads securely using local storage.

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
            file.save_as("", &file.name).await.unwrap();
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

### 13. Form Validation (`examples/13_validation.rs`)
Validate incoming request payloads declaratively with custom validation rules.

```rust
use flyer::{
    request::Request,
    response::Response,
    routing::next::Next,
    server,
    validation::Rules,
};

pub async fn register_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();
    rules.rule("email", vec!["required", "email"]);
    rules.rule("password", vec!["required", "min:5"]);
    return rules.handle(req, res, next).await;
}

pub async fn upload_documents_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();
    rules.rule("documents.*.name", vec!["required", "string", "min:5", "max:120"]);
    rules.rule("documents.*.file", vec!["required_with:files.*.name", "extensions:pdf,docx"]);
    return rules.handle(req, res, next).await;
}

pub async fn register(_req: Request, res: Response) -> Response {
    return res.html("<h1>Registration Successful!</h1>");
}

pub async fn upload(_req: Request, res: Response) -> Response {
    return res.html("<h1>Uploaded Documents Successful!</h1>");
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.post("register", register).middleware(register_form);
        router.post("documents", upload).middleware(upload_documents_form);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 14. WebSockets (`examples/14_websocket.rs`)
Build high-performance, real-time bidirectional communication channels.

```rust
use std::time::Duration;
use flyer::{server, websocket::Websocket};
use tokio::time::sleep;

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().ws("/", async |_req, ws| -> Websocket {
        let socket = ws.socket().clone();

        let ping_handler = tokio::spawn(async move {
            loop {
                println!("Pinging Client Every 5 seconds");
                sleep(Duration::from_secs(5)).await;
                socket.ping(Default::default()).await.unwrap();
            }
        });

        ws
            .keep_alive(ping_handler)
            .text(async |_payload, _socket| { todo!() })
            .binary(async |_payload, _socket| { todo!() })
            .ping(async |_payload, _socket| { todo!() })
            .pong(async |_payload, _socket| { todo!() })
            .close(async |_reason| { todo!() })
    });

    server.listen();
}
```

---

### 15. Mailer Integration (`examples/15_mail.rs`)
Send transactional emails out of the box using built-in mail utilities.

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
            .send("user@test.com".to_string(), Some("User"))
            .unwrap();

        return res.html("<h1>Email sent!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 16. Custom Server Hooks (`examples/16_hook.rs`)
Intercept request/response cycles before or after middleware and controllers.

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
    pub fn new() -> Self { Self { } }
}

impl Hook for CustomHook {
    async fn before(&self, req: Request, res: Response, next: Next) -> Response {
        println!("BEFORE HOOK");
        next.handle(req, res)
    }

    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        println!("AFTER HOOK");
        next.handle(req, res)
    }
}

pub fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().get("/", async |_req, res| {
        println!("CONTROLLER");
        res.html("<h1>Hello controller</h1>")
    });

    server.hook(CustomHook::new());
    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());
    server.listen();
}
```

---

### 17. Custom Error Loggers & Sentry (`examples/17_logger.rs`)
Define custom error logging behavior or integrate Sentry out of the box.

```rust
use flyer::{
    error::Error,
    loggers::{Logger, sentry::Sentry},
    request::Request,
    response::Response,
    server
};

pub struct DebuggerLogger { }

impl DebuggerLogger {
    pub fn new() -> Self { Self { } }
}

impl Logger for DebuggerLogger {
    async fn call(&self, error: Error, req: Request, _res: Response) -> () {
        println!("\r\n\r\nError: {}\r\nMessage: {}\r\nPath: {}\r\n\r\n", error.error, error.message, req.path());
    }
}

pub fn main() {
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
    // Prebuilt Sentry logger
    // server.logger(Sentry::new("SENTRY_DNS", "ENVIRONMENT"));

    println!("\r\n\r\nRunning Server: {}", server.address());
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
| `errors`    | Retrieves a validation error hash map for form fields.                    | `{{ error() }}`               |
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

---

## 🌐 Subdomain Local DNS Resolver Setup

For local testing of subdomain routing (`examples/` or multi-tenant apps), Flyer includes built-in local DNS utilities.

### Setup Instructions
* **macOS / Linux:**
  ```sh
  sudo bash -c 'echo -e "nameserver 127.0.0.1 \nport 5354" > /etc/resolver/tracker.com'
  ```
* **Windows (PowerShell):**
  ```powershell
  Add-DnsClientNrptRule -Namespace "tracker.com" -NameServers "127.0.0.1" -Comment "Per-domain DNS for tracker"
  ```
