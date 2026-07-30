use std::time::Duration;

use flyer::{
    loggers::{Logger, PanicErrorInfo, sentry::Sentry}, request::{Request, form::Form}, response::Response, routing::next::Next, server, server_tls, session::local::LocalSession, storage::{DEFAULT_STORAGE, local::LocalStorage}, validation::{Rules, Validator}, websocket::{Websocket, WriterInterface}
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
    println!("MIDDLEWARE 0");
    return next.handle(req, res);
}

pub async fn rule_exists(_form: &Form, _field: String, _args: Vec<String>) -> Option<String> {
    return None;
}

pub async fn upload(req: Request, res: Response) -> Response {
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
    async fn call(&self, info: PanicErrorInfo, req: Request, _res: Response) -> () {
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
        // let a: Option<String> = None;

        // a.unwrap();

        let mut validator = Validator::new(req.form(), {
            let mut rules = Rules::new();
            rules.rule("email", vec!["testing"]);
            rules
        });

        let valid = validator.validate().await;

        println!("\r\n\r\nVALIDATION -> {} : {:?}", valid, validator.errors());

        return res
            .view("index.html", None)
            .set_session("user_id", "10");
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




// Error: panicked at src/main.rs:57:11:
// called `Option::unwrap()` on a `None` value
// Message: called `Option::unwrap()` on a `None` value
// Path: /r

// thread 'tokio-rt-worker' (19644845) panicked at src/main.rs:57:11:
// called `Option::unwrap()` on a `None` value
// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace











































// use std::sync::Arc;
// use flyer::{hook::{Hook, HookErasure, HookWrapper}, request::Request, response::Response};

// // =========================================================================
// // FRAMEWORK CODE (100% clean public API, fix applied internally)
// // =========================================================================



// // 6. The Framework Manager
// pub struct Framework {
//     handlers: Vec<Box<dyn HookErasure>>,
// }

// impl Framework {
//     pub fn new() -> Self {
//         Self { handlers: Vec::new() }
//     }

//     pub fn register<T: Hook + 'static>(&mut self, instance: T) {
//         self.handlers.push(Box::new(HookWrapper {
//             instance: Arc::new(instance),
//         }));
//     }

//     // pub async fn run_all(&self, a: i32, b: i32) {
//     //     let mut handles = vec![];
        
//     //     for handler in &self.handlers {
//     //         let fut = handler.before(a, b);
//     //         let handle = tokio::spawn(fut); // Compiles perfectly now!
//     //         handles.push(handle);
//     //     }


//     //     for handle in handles {
//     //         let _ = handle.await;
//     //     }
//     // }
// }

// // =========================================================================
// // DEVELOPER APPLICATION CODE (Completely pristine, no macros, no errors)
// // =========================================================================

// struct MyStruct1 {
//     token: String,
// }

// impl MyStruct1 {
//     pub fn new(api_token: impl Into<String>) -> Self {
//         Self { token: api_token.into() }
//     }
// }

// impl Hook for MyStruct1 {
//     async fn before(&self, req: Request, res: Response) -> i32 {
//         todo!()
//     }
    
//     async fn after(&self, req: Request, res: Response) -> i32 {
//         todo!()
//     }
// }

// #[tokio::main]
// async fn main() {
//     let mut app = Framework::new();

//     app.register(MyStruct1::new("token_xyz_789"));

//     // app.run_all(50, 50).await;
// }