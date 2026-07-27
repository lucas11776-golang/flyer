use std::cell::RefCell;
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::future::{BoxFuture, join};
use futures::FutureExt;
use once_cell::sync::OnceCell;
use rustls::ServerConfig;
use tokio::runtime::Builder;

use crate::cookies::Cookies;
use crate::error::logger::{Logger, LoggerErasure, LoggerWrapper, PanicErrorInfo};
use crate::hooks::form::MultipartForm;
use crate::hooks::{Hook, HookErasure, HookWrapper};
use crate::request::Request;
use crate::response::Response;
use crate::routing::WebsocketHandler;
use crate::routing::next::Next;
use crate::routing::route::Route;
use crate::routing::{resolver::Resolver, router::Router, routes::Routes};
use crate::server::protocol::{tcp::Tcp, udp::Udp, ServerHandler};
use crate::session::local::LocalSession;
use crate::storage::{self, Storage};
use crate::utils::mem::Instance;
use crate::view::View;
use crate::websocket::Websocket;

pub(crate) mod protocol;

tokio::task_local! {
    pub(crate) static PANIC_CONTEXT: RefCell<PanicErrorInfo>;
}

pub(crate) static PANIC_IS_SET: OnceCell<()> = OnceCell::new();

pub(crate) type InitCallback = Box<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>;

pub struct Server {
    pub(crate) host: String,
    pub(crate) port: u32,
    pub(crate) routes: Routes,
    pub(crate) routers: Vec<Box<Router>>, 
    pub(crate) cookies: Box<dyn HookErasure>,
    pub(crate) session: Box<dyn HookErasure>,
    pub(crate) view: Box<dyn HookErasure>,
    pub(crate) multipart_form: Box<dyn HookErasure>,
    pub(crate) hooks: Vec<Box<dyn HookErasure>>,
    pub(crate) server_config: Option<ServerConfig>,
    pub(crate) loggers: Vec<Arc<dyn LoggerErasure + Send + Sync>>,
    pub(crate) init_callbacks: Vec<Arc<InitCallback>>,
}

impl Server {
    pub fn new(host: String, port: u32, server_config: Option<ServerConfig>) -> Self {
        return Self {
            host: host,
            port: port,
            routes: Routes::new(),
            routers: Vec::new(),
            cookies: Box::new(HookWrapper::new(Cookies::new())),
            session: Box::new(HookWrapper::new(LocalSession::new(Some("sessions"), Duration::from_secs(60 * 60)))),
            view: Box::new(HookWrapper::new(View::new(None))),
            multipart_form: Box::new(HookWrapper::new(MultipartForm::new())),
            hooks: Vec::new(),
            server_config: server_config,
            loggers: Vec::new(),
            init_callbacks: Vec::new(),
        };
    }

    pub fn address(&self) -> String {
        return format!("{}:{}", self.host, self.port);
    }

    pub fn router(&mut self) -> &mut Router {
        let instance = self.get_instance();
        self.routers.push(Box::new(Router::fresh(instance)));
        return self.routers.last_mut().unwrap();
    }

    pub fn session<H: Hook + 'static>(&mut self, hook: H) -> &mut Self {
        self.session = Box::new(HookWrapper::new(hook));
        return self;
    }

    pub fn view(&mut self, directory: impl Into<String>) -> &mut Self {
        self.view = Box::new(HookWrapper::new(View::new(Some(directory.into()))));
        return self;
    }

    pub fn listen(&mut self) {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.setup());
    }

    async fn setup(&mut self) {
        Resolver::new(self);
        return self.run().await;
    }

    pub fn error<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: Fn(PanicErrorInfo, Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.routes.errors.push(Box::new(move |err, req, res, next| {
            Box::pin(callback(err, req, res, next))
        }));
        return self;
    }

    pub fn hook<H: Hook + 'static>(&mut self, hook: H) -> &mut Self {
        self.hooks.push(Box::new(HookWrapper::new(hook)));
        return self;
    }

    pub fn logger<L: Logger + 'static>(&mut self, logger: L) -> &mut Self {
        self.setup_global_panic_hook();
        self.loggers.push(Arc::new(LoggerWrapper::new(logger)));
        return self;
    }

    pub fn storage<S: Storage + 'static>(&mut self, name: impl Into<String>, storage: S) -> &mut Self {
        storage::add(name, storage);
        self
    } 

    pub fn init<C, Fut>(&mut self, callback: C)
    where
        C: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self
            .init_callbacks
            .push(Arc::new(Box::new(move || Box::pin(callback()))));
    }

    async fn run(&mut self) {
        // if self.init_callback.is_some() {
        //     let instance = self.get_instance().clone();
        //     tokio::spawn(async move {
        //         instance.as_mut().init_callback.as_ref().unwrap()().await;
        //     });
        // }

        join(
            Udp::listen(self.get_instance()),
            Tcp::listen(self.get_instance()),
        )
        .await;
    }

    pub(crate) fn get_instance(&mut self) -> Instance<Server> {
        return Instance(self as *mut Self);
    }

    fn collect_hooks(&self, is_after: bool) -> Vec<&dyn HookErasure> {
        let mut hooks = Vec::with_capacity(4 + self.hooks.len());

        if is_after {
            hooks.push(self.multipart_form.as_ref());
            hooks.extend(self.hooks.iter().map(|h| h.as_ref()));
            hooks.push(self.session.as_ref());
            hooks.push(self.cookies.as_ref());
            hooks.push(self.view.as_ref());
        } else {
            hooks.push(self.cookies.as_ref());
            hooks.push(self.session.as_ref());
            hooks.push(self.multipart_form.as_ref());
            hooks.extend(self.hooks.iter().map(|h| h.as_ref()));
            hooks.push(self.view.as_ref());
        }

        return hooks;
    }

    async fn call_before_hooks(&self, mut req: Request, mut res: Response) -> (bool, Request, Response) {
        let hooks = self.collect_hooks(false);

        for hook in hooks {
            res.next(false);

            // let req_backup = req.clone();
            res = hook.before(req.clone(), res, Next::new()).await;

            if !res.is_next() {
                return (false, req, res);
            }

            req = res.request();
        }

        return (true, req, res);
    }

    async fn call_after_hooks(&self, mut req: Request, mut res: Response) -> (Request, Response) {
        let hooks = self.collect_hooks(true);

        for hook in hooks {
            res.next(false);

            res = hook.after(req.clone(), res, Next::new()).await;

            if !res.is_next() {
                return (req, res);
            }

            req = res.request();
        }

        return (req, res);
    }

    pub(crate) async fn on_http(&mut self, req: Request, mut res: Response) -> (Request, Response) {
        return PANIC_CONTEXT.scope(PanicErrorInfo::default().into(), async move {
            res.referer = req.header("referer");

            let req_backup = req.clone();
            let res_backup = res.clone();

            let result = AssertUnwindSafe(async {
                let (next, req, res) = self.call_before_hooks(req, res).await;

                if !next {
                    return (req, res);
                }

                let (req, res) = self.routes.handle_http(req, res).await;

                self.call_after_hooks(req, res).await
            })
            .catch_unwind()
            .await;


            if let Err(_) = result {
                let error = PANIC_CONTEXT.get().into_inner();
                self.on_logger(error.clone(), req_backup.clone(), res_backup.clone()).await;
                return self.routes.handle_error(error, req_backup, res_backup).await;
            }
            
            return result.unwrap();
        }).await;
    }

    pub(crate) async fn on_websocket(&self, req: Request, res: Response) -> (Request, Option<&Box<Route<WebsocketHandler>>>) {


        self
            .routes
            .handle_websocket(req, res)
            .await

        // return Ok(());

        // todo!()
    }

    pub(crate) async fn on_logger(&mut self, info: PanicErrorInfo, req: Request, res: Response) {
        for logger in &self.loggers {
            let logger_clone = Arc::clone(logger);
            let info_clone = info.clone();
            let req_clone = req.clone();
            let res_clone = res.clone();

            tokio::spawn(async move {
                logger_clone.call(info_clone, req_clone, res_clone).await;
            });
        }
    }

    pub fn setup_global_panic_hook(&self) {
        if PANIC_IS_SET.get().is_some() {
            return;
        }

        let _ = PANIC_IS_SET.set(panic::set_hook(Box::new(|info| {
            let result = PANIC_CONTEXT.try_with(|cell| {
                *cell.borrow_mut() = PanicErrorInfo::new(
                    info.to_string(),
                    info.payload_as_str().unwrap_or("").into(),
                );
            });

            if let Err(_) = result {
                // println!("Panic occurred outside of a PANIC_CONTEXT scope: {}", err);
            }
        })));
    }
}