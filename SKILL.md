---
name: flyer-framework
description: Comprehensive skill guide for AI agents to develop, refactor, and debug web applications using the Flyer framework for Rust.
---

# Flyer Web Framework Skill for AI Agents

## Overview & Purpose
Use this skill whenever you are tasked with designing, writing, refactoring, or debugging asynchronous web applications, APIs, microservices, or full-stack monoliths using the **Flyer** web framework for Rust.

Flyer is an asynchronous, high-performance Rust web framework running on Tokio. It provides Express/Laravel-style ergonomic abstractions (routing, middleware, Tera templating, sessions, file storage, validation, mailer, websockets) while preserving Rust's memory safety and concurrency guarantees.

---

## Capabilities & Architecture
* **Protocols:** HTTP/1.1, HTTP/2.0, HTTP/3.0, WebSockets.
* **Routing Engine:** Express-style route grouping, `{param}` path parameters, RESTful methods (`get`, `post`, `patch`, `delete`), and subdomain routing.
* **View Engine:** Native Tera templating with custom built-in functions for sessions, validation, flash messages, and environment variables.
* **State & Data:** Built-in session store, cookie manager, multipart form/file upload support (`LocalStorage`), and declarative validation rules.
* **Extensibility:** Middleware interceptors (`Next`), custom server request lifecycle hooks (`Hook`), and error logging / Sentry integration (`Logger`).

---

## Core Types & API Quick Reference

### Standard Handler Signatures
| Handler Type | Function Signature |
| :--- | :--- |
| **Standard Route** | `async fn(req: Request, res: Response) -> Response` |
| **Closure Route** | `async |_req, res| -> Response` |
| **Middleware** | `async fn(req: Request, res: Response, next: Next) -> Response` |
| **WebSocket** | `async |_req, ws| -> Websocket` |
| **Hook Lifecycle** | `async fn before(&self, req: Request, res: Response, next: Next) -> Response` |
| **Custom Logger** | `async fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> ()` |

### Key Framework Imports
```rust
use flyer::{
    server,
    request::Request,
    response::Response,
    routing::next::Next,
    view::ViewData,
    utils::{env, development::dns},
    hooks::{Hook, assets::AssetsHook},
    storage::local::LocalStorage,
    validation::Rules,
    websocket::{Websocket, Event, WriterInterface},
    mail::Mail,
    error::logger::{Logger, PanicErrorInfo},
    loggers::sentry::Sentry,
};
```

---

## Practical Code Patterns & Recipes

### 1. Server Initialization & Configuration
```rust
use std::time::Duration;
use flyer::{server, hooks::assets::AssetsHook, storage::local::LocalStorage, utils::env};

fn main() {
    env::load(".env");
    let host = env::env("HOST");
    let port: u32 = env::env("PORT").parse().unwrap_or(9999);

    let server = server(host, port)
        .view("views") // Register Tera template directory
        .storage("default", LocalStorage::new("storage"))
        .mailer("127.0.0.1".to_string(), 5555, "".to_string(), "".to_string(), false)
        .hook(AssetsHook::new("assets", Duration::from_secs(3600), 1024 * 10));

    server.router().get("/", async |_req, res| {
        res.html("<h1>Server Running</h1>")
    });

    server.listen();
}
```

### 2. Grouped Routing & Dynamic Path Parameters
```rust
use flyer::{server, request::Request, response::Response};

pub async fn get_user(req: Request, res: Response) -> Response {
    let user_id = req.parameter("user");
    res.html(format!("<h1>User ID: {}</h1>", user_id).as_str())
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/api", |router| {
        router.group("users", |router| {
            router.group("{user}", |router| {
                router.get("/", get_user);
            });
        });
    });

    server.listen();
}
```

### 3. Subdomain Routing & Local DNS Resolver
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

    // Spin up local development DNS resolver asynchronously alongside server
    server.init(async || {
        tokio::spawn(async {
            dns::run("tracker.com", "127.0.0.1", 5354);
        });
    });

    server.listen();
}
```

### 4. Middleware & Authentication Interceptors
```rust
use flyer::{
    server, request::Request, response::{Response, HTTP_UNAUTHORIZED}, routing::next::Next
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorMsg { message: String }

pub async fn auth_middleware(req: Request, res: Response, next: Next) -> Response {
    if req.header("authorization") != "my-secret-token" {
        return res.status_code(HTTP_UNAUTHORIZED).json(&ErrorMsg {
            message: "Unauthorized".into()
        });
    }
    next.handle(req, res).await
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("admin", |router| {
        router.get("/dashboard", async |_req, res| res.html("<h1>Admin Zone</h1>"));
    }).middleware(auth_middleware);

    server.listen();
}
```

### 5. Templating with Tera & `ViewData`
```rust
use flyer::{server, view::ViewData};
use serde::Serialize;

#[derive(Serialize)]
struct Profile { name: &'static str, role: &'static str }

fn main() {
    let server = server("127.0.0.1", 9999).view("views");

    server.router().get("/profile", async |_req, res| {
        let mut data = ViewData::new();
        data.insert("user", &Profile { name: "Alice", role: "Developer" });
        res.view("profile.html", Some(data))
    });

    server.listen();
}
```

### 6. Session & Cookie State Management
```rust
use std::time::Duration;
use flyer::{request::Request, response::Response, server};

pub async fn home_view(req: Request, mut res: Response) -> Response {
    // Read session variable
    let user_id = req.session().get("user_id").unwrap_or_default();
    
    // Set cookie with expiration
    res.cookies()
        .set("session_active", "true")
        .set_expires(Duration::from_secs(3600));

    res.html(format!("<h1>Welcome back, user #{}</h1>", user_id).as_str())
}

pub async fn login(_req: Request, res: Response) -> Response {
    // Set session variable and redirect back
    res.set_session("user_id", "42").back()
}
```

### 7. Form Validation & File Uploads
```rust
use flyer::{
    server, request::Request, response::Response, routing::next::Next, validation::Rules
};

pub async fn validate_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();
    rules.rule("email", vec!["required", "email"]);
    rules.rule("password", vec!["required", "min:5"]);
    rules.handle(req, res, next).await
}

pub async fn handle_upload(req: Request, res: Response) -> Response {
    if let Some(file) = req.file("avatar") {
        let _saved_path = file.save("uploads").await.unwrap();
        return res.html("<h1>Upload success</h1>");
    }
    res.html("<h1>No file uploaded</h1>")
}
```

### 8. WebSockets Implementation
```rust
use flyer::{server, websocket::{Websocket, Event, WriterInterface}};

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().ws("/chat", async |_req, ws| -> Websocket {
        ws.on(async |event, writer| {
            if let Event::Text(msg) = event {
                let _ = writer.write(format!("Echo: {}", msg));
            }
        })
    });

    server.listen();
}
```

### 9. Custom Lifecycle Hooks & Custom Error Logging
```rust
use flyer::{
    hooks::Hook,
    error::logger::{Logger, PanicErrorInfo},
    request::Request,
    response::Response,
    routing::next::Next,
    server
};

pub struct AuditHook;
impl Hook for AuditHook {
    async fn before(&self, req: Request, res: Response, next: Next) -> Response {
        println!("Incoming Request: {}", req.path());
        next.handle(req, res).await
    }
    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        println!("Request Processed");
        next.handle(req, res).await
    }
}

pub struct CustomLogger;
impl Logger for CustomLogger {
    async fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> () {
        println!("Error: {} | Message: {} | Path: {}", info.error, info.message, req.path());
    }
}
```

---

## Tera Template Helper Reference

| Helper Function | Arguments | Description | Example |
| :--- | :--- | :--- | :--- |
| `session` | `name` | Retrieve session value by key | `{{ session(name="user_id") }}` |
| `session_has` | `name` | Check if session key exists | `{% if session_has(name="user_id") %}` |
| `errors` |  | Retrieve hash map validation errors | `{{ errors() }}` |
| `error` | `name` | Retrieve field validation error | `{{ error(name="email") }}` |
| `error_has` | `name` | Check if field has validation error | `{% if error_has(name="email") %}` |
| `old` | `name` | Retrieve previously submitted form value | `value="{{ old(name="email") }}"` |
| `flash` | `name` | Get flash message | `{{ flash(name="success") }}` |
| `flash_has` | `name` | Check if flash message exists | `{% if flash_has(name="success") %}` |
| `env` | `name` | Read environment variable | `{{ env(name="APP_NAME") }}` |
| `url` | `path` | Generate absolute URL path | `{{ url(path="/login") }}` |

---

## Agent Guidance & Common Pitfalls

1. **Response Mutability for Cookie Setting:**
   When mutating response cookies in a handler, mark `res` as `mut`:
   `pub async fn home_view(_req: Request, mut res: Response) -> Response`

2. **Middleware Continuation (`next.handle`):**
   Middleware MUST call `next.handle(req, res).await` to pass execution down the chain. Failing to await or call `next.handle` will halt request processing.

3. **Route Parameter Extraction:**
   Dynamic route placeholders use single braces `{param}` (e.g., `router.group("{user}", ...)`). Retrieve parameters on the request object using `req.parameter("user")`.

4. **Tera Function Syntax:**
   Tera helpers strictly require named arguments with quotes (e.g., `session(name="key")`, not `session("key")`).

5. **Server Lifecycle Hooks (`Hook` trait):**
   Hook implementations require implementing both `before` and `after` methods and calling `next.handle(req, res).await`.

6. **Async Setup with `init`:**
   When running auxiliary async background tasks (like dev DNS server), pass an async closure to `server.init(async || { ... })` prior to calling `server.listen()`.