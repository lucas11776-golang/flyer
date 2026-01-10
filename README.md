# Flyer - Web Framework - supports HTTP/1.1, HTTP/2.0 and HTTP/3.0


## Information

Flyer web framework support concurrent request allowing you to run request without blocking each other.

## Getting Started

### Prerequisites

**Http key features:**

- Router
- Subdomain
- View
- Env
- Assets
- Middleware
- Session
- Cookie
- Multipart Form
- Form Validation
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

Insert code below in `main.rs`.

```rs
use flyer::{server, request::Request, response::Response};

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html("<h1>Users List</h1>");
}

pub async fn store<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.redirect("users/1");
}

pub async fn view<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.html(format!("<h1>User {}</h1>", req.parameter("user")).as_str());
}

pub async fn update<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.redirect(format!("users/{}", req.parameter("user")).as_str());
}

pub async fn destroy<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.redirect("users")
}

fn main() {
    let mut server = server("127.0.0.1", 9999);
    
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

### Subdomain

In order to use subdomain locally you have to edit you hosts DNS resolver file.

Windows: C:\Windows\System32\drivers\etc\hosts
MacOS: /etc/hosts
Linux: /etc/hosts

```hosts
127.0.0.1       localhost
255.255.255.255 broadcasthost
::1             localhost

# Custom Hosts Redirector
127.0.0.1 tracker.com
127.0.0.1 api.tracker.com
127.0.0.1 gentech.tracker.com
127.0.0.1 gentech.accounts.10.tracker.com
```

Insert code below in `main.rs`.

```rs
use flyer::server;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ApiInfo<'a> {
    info: &'a str,
    version: i32
}

fn main() {
    let mut server = server("127.0.0.1", 80);

    server.router().get("/", async |_req, res| {
        return res.html("<h1>Home Page</h1>");
    });

    server.router().subdomain("api", |router| {
        router.get("/", async  |_req, res| {
            return res.json(&ApiInfo {
                info: "Application details",
                version: 1
            });
        });
    });

    server.router().subdomain("{client}", |router| {
        router.get("/", async |req, res| {
            return res.html(format!("<h1>Client Name {}</h1>", req.parameter("client")).as_str());
        });
    });

    server.router().subdomain("{client}.accounts.{account_id}", |router| {
        router.get("/", async |req, res| {
            return res.html(format!("<h1>Client Name {} Account {}</h1>", req.parameter("client"), req.parameter("account_id")).as_str());
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```

Now you can start you server using

```sh
cargo run
```

MacOs may require use sudo to run on port 80.

```sh
sudo cargo run
```

after running you server you can visit:

[tracker.com](http://tracker.com)
[api.tracker.com](http://api.tracker.com)
[gentech.tracker.com](http://gentech.tracker.com)
[gentech.accounts.10.tracker.com](http://gentech.accounts.10.tracker.com)


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

### Env

Create file called `.env` and copy the content.

```env
APP_URL="http://127.0.0.1:9999/"

HOST="127.0.0.1"
PORT="9999"
```

Create file called `env.html` in folder called `views` and copy the content below in the file.

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <base href="{{ url() }}">
    <title>Running App On {{ env(name="APP_URL") }}</title>
</head>
<body>
    <h1>Hello Server: {{ url(path="/") }}</h1>
</body>
</html>
```

The next step to insert code below in `main.rs`.

```rust
use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    server,
    session::cookie::new_session_manager,
    utils::{env, load_env}
};

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("env.html", None);
}

fn main() {
    load_env(".env");

    let mut server = server(env("HOST").as_str(), env("PORT").parse().unwrap())
        .session(new_session_manager(Duration::from_hours(2), "cookie_token", "test_123"))
        .view("views");

    server.router().group("/", |router| {
        router.get("/", index);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


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
use std::time::Duration;

use flyer::{server, view::view_data};

fn main() {
    let mut server = server("127.0.0.1", 8888)
        .assets("assets", 1024, Duration::from_hours(2).as_millis())
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
use flyer::{
    request::Request,
    response::Response,
    router::next::Next,
    server
};
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

pub async fn verified<'a>(req: &'a mut Request, res: &'a mut Response, next: &mut Next) -> &'a mut Response {
    // Some logic to check user in database
    return next.handle(res);
}

fn main() {
    let mut server = server("127.0.0.1", 9999);

    server.router().group("api", |router| {
        router.group("users", |router| {
            router.group("{user}", |router| {
                router.get("/", async |req, res| {
                    return res.json(&User{
                        id: req.parameter("user").parse().unwrap(),
                        email: "joe@deo.com".to_owned()
                    });
                }).middleware(verified);
            });
        });
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
    router::next::Next,
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
        router.get("/", home_view).middleware(auth);
        router.get("register", register).middleware(guest);
        router.get("login", login).middleware(guest);
        router.get("logout", logout).middleware(auth);
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


### Multipart-Form

Create file called `index.html` in folder called views and copy the content below in the file.

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <base href="http://127.0.0.1:9999/">
  <title>Upload File</title>
  <style>
    body {
      text-align: center !important;
    }
  </style>
</head>
<body>
  <nav>
    <h1>Upload File</h1>
  </nav>
  <hr>
  <form method="post" action="/upload" enctype="multipart/form-data">
    <p style="color: red;">{{ error(name="file") }}</p>
    <p style="color: red;">{{ error_has(name="file") }}</p>
    <p style="color: red;">{{ error_has(name="file", class="is-invalid") }}</p>
    <input type="file" name="file" placeholder="Image">
    <br>
    <br>
    <br>
    <button type="submit">Upload File</button>
  </form>
</body>
</html>
```

The next step to insert code below in `main.rs`.

```rs
use std::{fs::File, io::Write, time::Duration};

use flyer::{
    request::Request,
    response::Response,
    server, 
    session::cookie::new_session_manager,
    view::view_data
};

pub async fn home<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("index.html", Some(view_data()));
}

pub async fn upload<'a>(req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    if req.file("file").is_none() {
        return res.with_error("file", "The file is required.")
            .back();
    }

    let uploaded = req.file("file").unwrap();
    let mut file = File::create(uploaded.name.as_str()).unwrap();

    file.write(&uploaded.content).unwrap();

    return res.redirect("/");
}

fn main() {
    let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"))
        .view("views");

    server.router().group("/", |router| {
        router.get("/", home);
        router.post("upload", upload);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Form Validation

Create file called `register.html` in folder called views and copy the content below in the file.

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <base href="http://127.0.0.1:9999/">
    <title>Register</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH" crossorigin="anonymous">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
            font-family: 'Poppins', sans-serif;
        }

        body {
            background-image: url(https://images.unsplash.com/photo-1542273917363-3b1817f69a2d?q=80&w=1748&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D);
            background-size: cover;
            background-position: center;
            background-repeat: no-repeat;
        }

        .login-form {
            background-color: #fff;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
            /* border-radius: 20px; */
            border-top-left-radius: 20px;
            border-bottom-left-radius: 20px;
        }

        .image-section {
            position: relative;
            background-image: url('https://images.unsplash.com/photo-1465146344425-f00d5f5c8f07?q=80&w=1752&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D');
            background-size: cover;
            background-position: center;
            border-top-right-radius: 20px;
            border-bottom-right-radius: 20px;
        }
        
        .icon {
            position: absolute;
            width: 60px;
            height: 60px;
            border-radius: 50%;
            display: flex;
            justify-content: center;
            align-items: center;
            background-color: #fff;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }

        .icon i {
            font-size: 24px;
        }

        .slack-icon {
            top: 40px;
            right: 60px;
            background-color: white;
        }

        .slack-icon i {
            color: #4A154B;
        }

        .user-icon {
            bottom: 200px;
            left: -30px;
            background-color: #fecaca;
        }

        .user-icon i {
            color: #dc2626;
        }
    </style>
    <base href="http://localhost:9999/">
</head>
<body class="d-flex justify-content-center align-items-center vh-100">
    <div class="container d-flex justify-content-center">
        <div class="row" style="width: 1000px;">
            <div class="col-md-6 d-flex flex-column p-5 login-form">
                <h1>Register</h1>
                <p class="text-muted">See your growth and get consulting support!</p>
                <form action="/register" method="post">
                    <div class="mb-3">
                        <label for="email" class="form-label">Email*</label>
                        <input type="email"
                               class="form-control {{ error_has(name="email", class="is-invalid") }}"
                               name="email"
                               value="{{ old(name="email") }}">
                        <div class="invalid-feedback">{{ error(name="email") }}</div>
                    </div>
                    <div class="mb-3">
                        <label for="password" class="form-label">Password*</label>
                        <input type="password"
                               class="form-control {{ error_has(name="password", class="is-invalid") }}"
                               name="password"
                               id="password">
                        <div class="invalid-feedback">{{ error(name="password") }}</div>
                    </div>
                    <div class="mb-3">
                        <label for="password_confirmatiom" class="form-label">Password Confirmation*</label>
                        <input type="password"
                               class="form-control {{ error_has(name="password_confirmatiom", class="is-invalid") }}"
                               name="password_confirmatiom"
                               id="password_confirmatiom">
                        <div class="invalid-feedback">{{ error(name="password_confirmatiom") }}</div>
                    </div>
                    <button type="submit" class="btn btn-primary w-100 py-2">Login</button>
                </form>
                <button class="btn mt-4 btn-outline-secondary d-flex align-items-center justify-content-center gap-2 w-100 mb-4">
                    <i class="fab fa-google text-danger"></i>
                    Sign in with Google
                </button>
            </div>
            <div class="col-md-6 image-section">
                <div class="icon slack-icon">
                    <i class="fab fa-slack"></i>
                </div>
                <div class="icon user-icon">
                    <i class="fas fa-user"></i>
                </div>
            </div>
        </div>
    </div>
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous"></script>
</body>
</html>
```

The next step to insert code below in `main.rs`.

```rust
use std::time::Duration;

use serde::{Deserialize, Serialize};

use flyer::{
    request::{Request, form::Form},
    response::Response,
    router::next::Next,
    server,
    session::cookie::new_session_manager,
    validation::{Rules, Validator, rules}
};
use tokio::time::sleep;

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub token: String,
    pub r#type: String,
    pub expires: u128
}

pub async fn index<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.view("register.html", None);
}

pub async fn login<'a>(_req: &'a mut Request, res: &'a mut Response) -> &'a mut Response {
    return res.json(&Token {
        token: String::from("eye.jwt.token"),
        r#type: String::from("jwt"),
        expires: Duration::from_hours(24).as_millis()
    });
}

pub async fn email_exists(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let users_table = vec!["jeo@doe.com", "jane@deo.com"];

    sleep(Duration::from_millis(250)).await; // Database call simulation 

    for email in users_table {
        if form.values.get(&field).unwrap().eq(email) {
            return Some(format!("The {} already exists in database", field))
        }
    }

    return None
}

async fn login_form<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    let mut rules = Rules::new();

    rules.field("email")
        .add(rules::required, vec![])
        .add(rules::string, vec![])
        .add(email_exists, vec![]);
    rules.field("password")
        .add(rules::required, vec![])
        .add(rules::string, vec![])
        .add(rules::min, vec!["8"])
        .add(rules::max, vec!["21"])
        .add(rules::confirmed, vec![]);

    return Validator::handle(req, res, next, rules).await;
}

fn main() {
    let mut server = server("127.0.0.1", 9999)
        .session(new_session_manager(Duration::from_hours(2), "session_cookie_key_name", "encryption"))
        .view("views")
        .assets("assets", 1024, Duration::from_hours(2).as_millis());

    server.router().group("/", |router| {
        router.get("/", index);
        router.group("register", |router| {
            router.post("/", login).middleware(login_form);
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```


### Websocket

Insert code below in `main.rs`.

```rust
use flyer::router::next::Next;
use flyer::{server};
use flyer::{request::Request, response::Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<'a> {
    message: &'a str
}

pub async fn auth<'a>(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
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
        });

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
        }).middleware(auth);
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}
```