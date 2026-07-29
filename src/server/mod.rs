use std::cell::RefCell;
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::Duration;

use futures::future::{join, BoxFuture};
use futures::FutureExt;
use once_cell::sync::OnceCell;
use rustls::ServerConfig;
use tokio::runtime::Builder;

use crate::cookies::Cookies;
use crate::error::logger::{Logger, LoggerErasure, LoggerWrapper, PanicErrorInfo};
use crate::hooks::form::MultipartFormHook;
use crate::hooks::{Hook, HookErasure, HookWrapper};
use crate::mail;
use crate::request::Request;
use crate::response::Response;
use crate::routing::next::Next;
use crate::routing::resolver::Resolver;
use crate::routing::router::Router;
use crate::routing::routes::Routes;
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

pub(crate) type InitCallback = dyn Fn() -> BoxFuture<'static, ()> + Send + Sync;

pub struct Server {
    pub(crate) host: String,
    pub(crate) port: u32,
    pub(crate) routes: Routes,
    pub(crate) routers: Vec<Router>,
    pub(crate) cookies: Arc<dyn HookErasure>,
    pub(crate) session: Arc<dyn HookErasure>,
    pub(crate) view: Arc<dyn HookErasure>,
    pub(crate) multipart_form: Arc<dyn HookErasure>,
    pub(crate) hooks: Vec<Arc<dyn HookErasure>>,
    pub(crate) server_config: Option<ServerConfig>,
    pub(crate) loggers: Vec<Arc<dyn LoggerErasure + Send + Sync>>,
    pub(crate) init_callbacks: Vec<Arc<InitCallback>>,
    pub(crate) before_hooks: Vec<Arc<dyn HookErasure>>,
    pub(crate) after_hooks: Vec<Arc<dyn HookErasure>>,
}

impl Server {
    pub fn new(host: String, port: u32, server_config: Option<ServerConfig>) -> Self {
        Self {
            host,
            port,
            routes: Routes::new(),
            routers: Vec::new(),
            cookies: Arc::new(HookWrapper::new(Cookies::new())),
            session: Arc::new(HookWrapper::new(LocalSession::new(Some("sessions"), Duration::from_secs(3600),))),
            view: Arc::new(HookWrapper::new(View::new(None::<String>))),
            multipart_form: Arc::new(HookWrapper::new(MultipartFormHook::new())),
            hooks: Vec::new(),
            server_config,
            loggers: Vec::new(),
            init_callbacks: Vec::new(),
            before_hooks: Vec::new(),
            after_hooks: Vec::new(),
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn router(&mut self) -> &mut Router {
        let instance = self.get_instance();
        self.routers.push(Router::fresh(instance));
        self.routers.last_mut().unwrap()
    }

    pub fn session<H: Hook + 'static>(&mut self, hook: H) -> &mut Self {
        self.session = Arc::new(HookWrapper::new(hook));
        self
    }

    pub fn view(&mut self, directory: impl Into<String>) -> &mut Self {
        self.view = Arc::new(HookWrapper::new(View::new(Some(directory.into()))));
        self
    }

    pub fn listen(&mut self) {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.setup());
    }

    async fn setup(&mut self) {
        self.prepare_hooks();
        Resolver::new(self);
        self.run().await;
    }

    fn prepare_hooks(&mut self) {
        let extra = self.hooks.len();

        let mut before = Vec::with_capacity(4 + extra);
        before.push(Arc::clone(&self.cookies));
        before.push(Arc::clone(&self.session));
        before.push(Arc::clone(&self.multipart_form));
        before.extend(self.hooks.iter().cloned());
        before.push(Arc::clone(&self.view));

        let mut after = Vec::with_capacity(4 + extra);
        after.push(Arc::clone(&self.multipart_form));
        after.extend(self.hooks.iter().cloned());
        after.push(Arc::clone(&self.session));
        after.push(Arc::clone(&self.cookies));
        after.push(Arc::clone(&self.view));

        self.before_hooks = before;
        self.after_hooks = after;
    }

    pub fn error<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: Fn(PanicErrorInfo, Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.routes.errors.push(Box::new(move |err, req, res, next| {
            Box::pin(callback(err, req, res, next))
        }));
        self
    }

    pub fn hook<H: Hook + 'static>(&mut self, hook: H) -> &mut Self {
        self.hooks.push(Arc::new(HookWrapper::new(hook)));
        self
    }

    pub fn logger<L: Logger + 'static>(&mut self, logger: L) -> &mut Self {
        self.setup_global_panic_hook();
        self.loggers.push(Arc::new(LoggerWrapper::new(logger)));
        self
    }

    pub fn storage<S: Storage + 'static>(&mut self, name: impl Into<String>, storage: S) -> &mut Self {
        storage::add(name, storage);
        self
    }

    pub fn mailer(
        &mut self,
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
        tls: bool,
    ) -> &mut Self {
        mail::SMTP::init(host, port, username, password, tls).unwrap();
        self
    }

    pub fn init<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.init_callbacks
            .push(Arc::new(move || Box::pin(callback())));
        self
    }

    async fn run(&mut self) {
        for init in &self.init_callbacks {
            let init_cb = Arc::clone(init);
            tokio::spawn(async move {
                init_cb().await;
            });
        }

        join(
            Udp::listen(self.get_instance()),
            Tcp::listen(self.get_instance()),
        )
        .await;
    }

    pub(crate) fn get_instance(&mut self) -> Instance<Server> {
        Instance(self as *mut Self)
    }

    async fn call_before_hooks(&self, mut req: Request, mut res: Response) -> (bool, Request, Response) {
        for hook in &self.before_hooks {
            res.next(false);
            res = hook.before(req, res, Next::new()).await;

            if !res.is_next() {
                return (false, res.request(), res);
            }

            req = res.request();
        }

        (true, req, res)
    }

    async fn call_after_hooks(&self, mut req: Request, mut res: Response) -> (Request, Response) {
        for hook in &self.after_hooks {
            res.next(false);
            res = hook.after(req, res, Next::new()).await;

            if !res.is_next() {
                return (res.request(), res);
            }

            req = res.request();
        }

        (req, res)
    }

    pub(crate) async fn on_http(&self, req: Request, mut res: Response) -> (Request, Response) {
        PANIC_CONTEXT.scope(RefCell::new(PanicErrorInfo::default()), async move {
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

            match result {
                Ok(out) => out,
                Err(_) => {
                    let error = PANIC_CONTEXT.with(|cell| cell.borrow().clone());
                    self.on_logger(error.clone(), req_backup.clone(), res_backup.clone()).await;
                    self.routes.handle_error(error, req_backup, res_backup).await
                }
            }
        }).await
    }

    pub(crate) async fn on_websocket(&self, req: Request, res: Response) -> Option<Websocket> {
        PANIC_CONTEXT.scope(RefCell::new(PanicErrorInfo::default()), async move {
            let result = AssertUnwindSafe(async {
                let (req, res, route) = self
                    .routes
                    .handle_websocket(req.clone(), res.clone())
                    .await;


                if route.is_none() {
                    return (req, res, None)
                }

                (req, res, route)
            })
            .catch_unwind()
            .await;

            match result {
                Ok((req, _, route)) => {
                    return Some((route.unwrap().handler)(req, Websocket::new()).await)
                },
                Err(_) => {
                    let error = PANIC_CONTEXT.with(|cell| cell.borrow().clone());
                    self.on_logger(error.clone(), req.clone(), res.clone()).await;
                    self.routes.handle_error(error, req, res).await;
                    return None
                },
            }
        }).await
    }

    pub(crate) async fn on_logger(&self, info: PanicErrorInfo, req: Request, res: Response) {
        for logger in &self.loggers {
            let logger = Arc::clone(logger);
            let info = info.clone();
            let req = req.clone();
            let res = res.clone();

            tokio::spawn(async move {
                logger.call(info, req, res).await;
            });
        }
    }

    pub fn setup_global_panic_hook(&self) {
        PANIC_IS_SET.get_or_init(|| {
            panic::set_hook(Box::new(|info| {
                let _ = PANIC_CONTEXT.try_with(|cell| {
                    *cell.borrow_mut() = PanicErrorInfo::new(
                        info.to_string(),
                        info.payload_as_str().unwrap_or("").into(),
                    );
                });
            }));
        });
    }
}