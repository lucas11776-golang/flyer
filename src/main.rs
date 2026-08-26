use std::time::Duration;

use flyer::{
    error::Error,
    loggers::Logger,
    request::{Request, form::Form},
    response::Response,
    routing::next::Next,
    server,
    session::local::LocalSession,
    storage::{DEFAULT_STORAGE, local::LocalStorage},
    validation::{Rules, Validator},
    websocket::{Websocket, WriterInterface}
};

pub async fn http(_req: Request, res: Response) -> Response {
    return res.body("<h1>Hello World</h1>".as_bytes());
}

pub async fn group(req: Request, res: Response, next: Next) -> Response {
    println!("MIDDLEWARE Group");
    return next.handle(req, res);
}

pub async fn middleware(_req: Request, _res: Response, _next: Next) -> Response {
    return _next.handle(_req, _res);
}

pub async fn middleware1(req: Request, res: Response, next: Next) -> Response {
    return next.handle(req, res);
}

pub async fn rule_exists(_form: &Form, _field: String, _args: Vec<String>) -> Option<String> {
    return None;
}

pub async fn upload(req: Request, res: Response) -> Response {
    let mut validator = Validator::new(req.form(), {
        let mut rules = Rules::new();

        rules.rule("files.*.title", vec!["required", "string"]);
        rules.rule("files.*.file", vec!["required_with:files.*.title", "image"]);

        rules
    });

    let result = validator.validate().await;


    println!("VALIDATION ---> {}", result);
    println!("ERRORS ---> {:?}", validator.errors());


    if req.files().len() > 0 {
        for (_, file) in req.files() {
            file
                .save_as("", &file.name)
                .await
                .unwrap();
        }
        return res.html("<h1>File uploaded!</h1>");
    }
    return res.html("<h1>No file uploaded!</h1>");
}

pub struct DebuggerLogger { }

impl DebuggerLogger {
    pub fn new() -> Self {
        Self { }
    }
}

impl Logger for DebuggerLogger {
    async fn call(&self, info: Error, _req: Request, _res: Response) -> () {
        println!("{}", info);
    }
}

pub fn main() {
    let server = server("127.0.0.1", 9999)
    // let server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
        .view("views")
        .session(LocalSession::new(Some("sessions"), Duration::from_secs(60 * 60)))
        .storage(DEFAULT_STORAGE, LocalStorage::new("storage"));


    Rules::add("testing", rule_exists);

    server.router().get("/", async |req, res| {
        // // let a: Option<String> = None;

        // // a.unwrap();

        // let mut validator = Validator::new(req.form(), {
        //     let mut rules = Rules::new();
        //     rules.rule("email", vec!["testing"]);
        //     rules
        // });

        // let valid = validator.validate().await;

        // println!("\r\n\r\nVALIDATION -> {} : {:?}", valid, validator.errors());

        // return res
        //     .view("index.html", None)
        //     .set_session("user_id", "10");




        let html = String::from("<h1>Hello World</h1>");


        let res = res.set_header("Content-Length", html.len().to_string());

        res.write(html.into()).unwrap();


        res
    });

    server.router().post("upload", upload);


    server.router().ws("/", async |_req, ws| -> Websocket {
        ws.on(async |event, writer| {
            match event {
                flyer::websocket::Event::Ready() => todo!(),
                flyer::websocket::Event::Text(_items) => {
                    writer.write("HELLO TO YOU".into()).unwrap()
                },
                flyer::websocket::Event::Binary(_items) => todo!(),
                flyer::websocket::Event::Ping(_items) => todo!(),
                flyer::websocket::Event::Pong(_items) => todo!(),
                flyer::websocket::Event::Close(_reason) => todo!(),
            }
        })
    });

    server.router().get("/submit", async |_req, res| {
        return res.back();
    });

    server.router().group("api", |router| {
        router.group("v1", |router| {
            router.group("users", |router| {
                router.get("/", http).middleware(middleware);
            }).middleware(group);
        }); //.middleware(group);
    });

    server.init(async || {

    });

    server.logger(DebuggerLogger::new());

    // server.logger(Sentry::new(
    //     "https://1ebec3a6d4b06d781b1040f3fce14f4f@o4511693601177600.ingest.us.sentry.io/4511693603930112",
    //     "DEVELOPMENT"
    // ));

    println!("\r\n\r\nRunning Server: {}\r\n\r\n", server.address());

    server.listen();
}