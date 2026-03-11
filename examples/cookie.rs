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