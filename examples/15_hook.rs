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