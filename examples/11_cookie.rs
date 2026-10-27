use std::time::Duration;
use flyer::{
    request::Request,
    response::Response,
    server,
};

pub async fn home_view(_req: Request, mut res: Response) -> Response {
    // Set a cookie
    res.cookies()
        .set("user_id", "1")
        .set_expires(Duration::from_secs(3600));

    return res.html("<h1>Cookie has been set! Visit <a href='/cookie'>/cookie</a></h1>");
}

pub async fn cookie(req: Request, res: Response) -> Response {
    // Get a cookie
    let cookie_val = req.cookies().get("user_id");
    return res.html(format!("<h1>User ID cookie is {:?}</h1>", cookie_val).as_str());
}

pub async fn remove_cookie(_req: Request, mut res: Response) -> Response {
    // Remove a cookie
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
