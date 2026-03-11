use std::time::Duration;

use flyer::{
    request::Request,
    response::Response,
    server,
    session::cookie::new_session_manager,
    utils::{env, load_env}
};

/*

TODO: Create file called `.env` and copy the content.

```env
APP_URL="http://127.0.0.1:9999/"

HOST="127.0.0.1"
PORT="9999"
```

TODO: Create file called `index.html` in folder called `views` and copy the content below in the file.

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

*/

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