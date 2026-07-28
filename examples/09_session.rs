use flyer::{
    request::Request,
    response::Response,
    server,
};

/// Session Example
///
/// This example demonstrates:
/// - Storing data in a user session
/// - Retrieving data from the session
/// - Managing login state
pub async fn home_view(req: Request, res: Response) -> Response {
    let user_id = req.session().get("user_id");
    if user_id == "" {
        return res.html("<h1>Not logged in</h1>");
    }
    return res.html(format!("<h1>Welcome user {}</h1>", user_id).as_str());
}

pub async fn login(_req: Request, res: Response) -> Response {
    // Set session data
    return res
        .set_session("user_id", "1")
        .back();
}

pub async fn logout(_req: Request, res: Response) -> Response {
    // Remove session data
    return res
        .remove_session("user_id")
        .back();
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
