use std::collections::HashMap;

use futures::future::BoxFuture;

use crate::error::logger::PanicErrorInfo;
use crate::request::Request;
use crate::response::Response;
use crate::routing::next::Next;
use crate::routing::router::Router;
use crate::websocket::Websocket;

pub mod next;
pub mod route;
pub mod router;
pub(crate) mod routes;
pub(crate) mod resolver;

pub type HttpHandler = Box<dyn Fn(Request, Response) -> BoxFuture<'static, Response> + Send + Sync>;

pub type HttpErrorHandler = Box<dyn Fn(PanicErrorInfo, Request, Response, Next) -> BoxFuture<'static, Response> + Send + Sync>;

pub type MiddlewareHandler = Box<dyn Fn(Request, Response, Next) -> BoxFuture<'static, Response> + Send + Sync>;

pub type Middlewares = HashMap<String, Box<MiddlewareHandler>>;

pub type WebsocketHandler = Box<dyn Fn(Request, Websocket) -> BoxFuture<'static, ()> + Send + Sync>;

pub type Group = for<'a> fn(&mut Router);