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

/*

TODO: Create file called `register.html` in folder called `views` and copy the content below in the file.

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

*/

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