use flyer::{
    request::Request,
    response::Response,
    routing::next::Next,
    server,
    validation::Rules,
};

pub async fn register_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();

    // Define rules
    rules.rule("email", vec!["required", "email"]);
    rules.rule("password", vec!["required", "min:5"]);

    // Handle rules; if invalid, this will automatically handle the failure
    return rules.handle(req, res, next).await;
}

pub async fn upload_documents_form(req: Request, res: Response, next: Next) -> Response {
    let mut rules = Rules::new();

    // Define rules
    rules.rule("documents.*.name", vec!["required", "string", "min:5", "max:120"]);
    rules.rule("documents.*.file", vec!["required_with:files.*.name", "extensions:pdf,docx"]);

    // Handle rules; if invalid, this will automatically handle the failure
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
