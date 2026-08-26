use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;
use serde::Serialize;

use crate::{
    cookies::{Cookies, cookie::Cookie}, request::Request, routing::next::Next, session::Session, utils::{Values, future::SendFuture, http::Headers, mem::Instance}, view::{ViewBag, ViewData},
};

pub type StatusCode = u16;

pub const HTTP_CONTINUE: StatusCode = 100;
pub const HTTP_SWITCHING_PROTOCOLS: StatusCode = 101;
pub const HTTP_PROCESSING: StatusCode = 102;
pub const HTTP_EARLY_HINTS: StatusCode = 103;
pub const HTTP_OK: StatusCode = 200;
pub const HTTP_CREATED: StatusCode = 201;
pub const HTTP_ACCEPTED: StatusCode = 202;
pub const HTTP_NON_AUTHORITATIVE_INFORMATION: StatusCode = 203;
pub const HTTP_NO_CONTENT: StatusCode = 204;
pub const HTTP_RESET_CONTENT: StatusCode = 205;
pub const HTTP_PARTIAL_CONTENT: StatusCode = 206;
pub const HTTP_MULTI_STATUS: StatusCode = 207;
pub const HTTP_ALREADY_REPORTED: StatusCode = 208;
pub const HTTP_IM_USED: StatusCode = 226;
pub const HTTP_MULTIPLE_CHOICES: StatusCode = 300;
pub const HTTP_MOVED_PERMANENTLY: StatusCode = 301;
pub const HTTP_FOUND: StatusCode = 302;
pub const HTTP_SEE_OTHER: StatusCode = 303;
pub const HTTP_NOT_MODIFIED: StatusCode = 304;
pub const HTTP_USE_PROXY: StatusCode = 305;
pub const HTTP_TEMPORARY_REDIRECT: StatusCode = 307;
pub const HTTP_PERMANENT_REDIRECT: StatusCode = 308;
pub const HTTP_BAD_REQUEST: StatusCode = 400;
pub const HTTP_UNAUTHORIZED: StatusCode = 401;
pub const HTTP_PAYMENT_REQUIRED: StatusCode = 402;
pub const HTTP_FORBIDDEN: StatusCode = 403;
pub const HTTP_NOT_FOUND: StatusCode = 404;
pub const HTTP_METHOD_NOT_ALLOWED: StatusCode = 405;
pub const HTTP_NOT_ACCEPTABLE: StatusCode = 406;
pub const HTTP_PROXY_AUTHENTICATION_REQUIRED: StatusCode = 407;
pub const HTTP_REQUEST_TIMEOUT: StatusCode = 408;
pub const HTTP_CONFLICT: StatusCode = 409;
pub const HTTP_GONE: StatusCode = 410;
pub const HTTP_LENGTH_REQUIRED: StatusCode = 411;
pub const HTTP_PRECONDITION_FAILED: StatusCode = 412;
pub const HTTP_CONTENT_TOO_LARGE: StatusCode = 413;
pub const HTTP_URI_TOO_LONG: StatusCode = 414;
pub const HTTP_UNSUPPORTED_MEDIA_TYPE: StatusCode = 415;
pub const HTTP_RANGE_NOT_SATISFIABLE: StatusCode = 416;
pub const HTTP_EXPECTATION_FAILED: StatusCode = 417;
pub const HTTP_IM_A_TEAPOT: StatusCode = 418;
pub const HTTP_MISDIRECTED_REQUEST: StatusCode = 421;
pub const HTTP_UNPROCESSABLE_CONTENT: StatusCode = 422;
pub const HTTP_LOCKED: StatusCode = 423;
pub const HTTP_FAILED_DEPENDENCY: StatusCode = 424;
pub const HTTP_TOO_EARLY: StatusCode = 425;
pub const HTTP_UPGRADE_REQUIRED: StatusCode = 426;
pub const HTTP_PRECONDITION_REQUIRED: StatusCode = 428;
pub const HTTP_TOO_MANY_REQUESTS: StatusCode = 429;
pub const HTTP_REQUEST_HEADER_FIELDS_TOO_LARGE: StatusCode = 431;
pub const HTTP_UNAVAILABLE_FOR_LEGAL_REASONS: StatusCode = 451;
pub const HTTP_INTERNAL_SERVER_ERROR: StatusCode = 500;
pub const HTTP_NOT_IMPLEMENTED: StatusCode = 501;
pub const HTTP_BAD_GATEWAY: StatusCode = 502;
pub const HTTP_SERVICE_UNAVAILABLE: StatusCode = 503;
pub const HTTP_GATEWAY_TIMEOUT: StatusCode = 504;
pub const HTTP_HTTP_VERSION_NOT_SUPPORTED: StatusCode = 505;
pub const HTTP_VARIANT_ALSO_NEGOTIATES: StatusCode = 506;
pub const HTTP_INSUFFICIENT_STORAGE: StatusCode = 507;
pub const HTTP_LOOP_DETECTED: StatusCode = 508;
pub const HTTP_NOT_EXTENDED: StatusCode = 510;
pub const HTTP_NETWORK_AUTHENTICATION_REQUIRED: StatusCode = 511;

#[derive(Clone)]
pub struct Response {
    pub(crate) next: Option<Next>,
    pub(crate) status_code: StatusCode,
    pub(crate) headers: Headers,
    pub(crate) referer: String,
    pub(crate) content: Bytes,
    pub(crate) cookies: Cookies,
    pub(crate) session: Session,
    pub view: Option<ViewBag>,
    // TODO: temp fix
    pub(crate) writer: Option<Arc<dyn WriterTErasure>>,
    pub(crate) is_next: bool,
    pub(crate) has_sent: bool,
}


#[allow(async_fn_in_trait)]
pub trait Writer: Send + Sync {
    async fn write(&self, data: Bytes) -> Result<()> ;
}


#[allow(async_fn_in_trait)]
pub trait WriterT: Send + Sync {
    async fn write(&self, data: Bytes) -> Result<()>;
}

pub(crate) trait WriterTErasure: Send + Sync {
    fn write(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
}

pub struct LoggerTWrapper<T: WriterT + 'static> {
    pub instance: Arc<T>,
    pub has_sent: bool,
}

impl <T: WriterT + 'static>LoggerTWrapper<T> {
    pub fn new(instance: T) -> Self {
        return Self {
            instance: Arc::new(instance),
            has_sent: false,
        };
    }
}

impl<T: WriterT + 'static> WriterTErasure for LoggerTWrapper<T> {
    fn write(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
        let instance = Arc::clone(&self.instance);
        // let res_instance = LOCAL_RESPONSE.get();
        // let res = res_instance.as_mut();

        // if !res.has_sent {
        //     res.has_sent = true
        // }

        return Box::pin(async move {
            SendFuture(instance.write(data)).await
        });
    }
}


tokio::task_local! {
    pub(crate) static LOCAL_RESPONSE: Instance<Response>;
}


impl Into<serde_json::Value> for Response {
    fn into(self) -> serde_json::Value {
        serde_json::json!({
            "status_code": &self.status_code,
            "headers": &self.headers,
            "cookies": &self.cookies,
            "session": &self.session,
            "view": &self.view,
        })
    }
}

impl Response {
    #[inline]
    // pub(crate) fn new(writer: impl WriterTErasure + 'static) -> Self {
    pub(crate) fn new() -> Self {
        Self {
            next: None,
            status_code: HTTP_OK,
            headers: Headers::new(),
            referer: String::from("/"),
            content: Bytes::new(),
            cookies: Default::default(),
            session: Default::default(),
            view: None,
            is_next: false,
            // writer: Arc::new(writer),
            writer: None,
            has_sent: false,

        }
    }

    #[inline]
    pub fn request(&mut self) -> Request {
        self.next
            .as_mut()
            .expect("Middleware chain error: Next is None")
            .request()
    }

    #[inline]
    pub fn status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn header(&self, key: &str) -> String {
        self.headers
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    #[inline]
    pub fn set_header(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.headers.insert(k.into(), v.into());
        self
    }

    pub fn set_headers(mut self, headers: Headers) -> Self {
        for (k, v) in headers {
            self.headers.insert(k, v);
        }
        self
    }

    #[inline]
    pub fn cookies(&mut self) -> &mut Cookies {
        &mut self.cookies
    }

    #[inline]
    pub fn set_cookie(&mut self, k: impl Into<String>, v: impl Into<String>) -> &mut Cookie {
        self.cookies.set(k, v)
    }

    #[inline]
    pub fn remove_cookie(mut self, k: impl Into<String>) -> Self {
        self.cookies.remove(k);
        self
    }

    #[inline]
    pub fn session(&mut self) -> &mut Session {
        &mut self.session
    }

    #[inline]
    pub fn set_session(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.session.set(k, v);
        self
    }

    #[inline]
    pub fn remove_session(mut self, k: impl Into<String>) -> Self {
        self.session.remove(k);
        self
    }

    #[inline]
    pub fn set_flash(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.session.set_flash(k, v);
        self
    }

    #[inline]
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.content = body.into();
        self
    }

    pub fn json<J>(self, object: &J) -> Self
    where
        J: ?Sized + Serialize,
    {
        match serde_json::to_vec(object) {
            Ok(bytes) => self
                .set_header("Content-Type", "application/json")
                .body(bytes),
            Err(_) => self.status_code(HTTP_INTERNAL_SERVER_ERROR),
        }
    }

    #[inline]
    pub fn html(self, html: impl Into<String>) -> Self {
        let html_str = html.into();
        self.set_header("Content-Type", "text/html; charset=utf-8")
            .body(html_str)
    }

    #[inline]
    pub fn view(mut self, view: &str, data: Option<ViewData>) -> Self {
        self.view = Some(ViewBag::new(view, data));
        self
    }

    #[inline]
    pub fn redirect(self, to: impl Into<String>) -> Self {
        self.redirect_with_status_code(to, HTTP_TEMPORARY_REDIRECT)
    }

    #[inline]
    pub fn redirect_permanent(self, to: impl Into<String>) -> Self {
        self.redirect_with_status_code(to, HTTP_PERMANENT_REDIRECT)
    }

    fn redirect_with_status_code(self, to: impl Into<String>, status_code: StatusCode) -> Self {
        let target = to.into();
        let html = format!(
            "<!DOCTYPE html><html><head><meta http-equiv=\"Refresh\" content=\"0; url='{target}'\"></head><body></body></html>"
        );

        self
            .html(html)
            .status_code(status_code)
    }

    pub fn back(self) -> Self {
        if self.referer.is_empty() {
            self.redirect("/")
        } else {
            let referer = self.referer.clone();
            self.redirect(referer)
        }
    }

    pub fn with_error(mut self, name: impl Into<String>, error: impl Into<String>) -> Self {
        self.session.errors.insert(name.into(), error.into());
        self
    }

    pub fn with_errors(mut self, errors: Values) -> Self {
        for (name, error) in errors {
            self.session.errors.insert(name, error);
        }
        self
    }

    ///
    /// 
    /// 
    pub async fn write(&self, data: Bytes) -> Result<()> {
        self
            .writer
            .as_ref()
            .unwrap()
            .write(data)
            .await
    }

    pub fn with_flash(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.session.flash.insert(key.into(), value.into());
        self
    }

    pub(crate) fn with_old(mut self, old: Values) -> Self {
        for (k, v) in old {
            self.session.old.insert(k, v);
        }
        self
    }

    #[inline]
    pub(crate) fn next(&mut self, is: bool) {
        self.is_next = is;
    }

    #[inline]
    pub(crate) fn is_next(&self) -> bool {
        self.is_next
    }
}