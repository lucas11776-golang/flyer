use flyer::{
    error::logger::{Logger, PanicErrorInfo},
    loggers::sentry::Sentry,
    request::Request,
    response::Response, server
};

pub struct DebuggerLogger { }

impl DebuggerLogger {
    pub fn new() -> Self {
        Self { }
    }
}

impl Logger for DebuggerLogger {
    async fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> () {
        println!("\r\n\r\nError: {}\r\nMessage: {}\r\nPath: {}r\n\r\n", info.error, info.message, req.path());
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

    // Prebuilt logger for sentry
    // server.logger(Sentry::new("SENTRY_DNS", "ENVIRONMENT"));

    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());

    server.listen();
}