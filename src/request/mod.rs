use std::{collections::HashMap, net::{IpAddr, SocketAddr}};

use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    cookies::Cookies,
    request::form::{File, Form},
    server::Server,
    session::Session,
    utils::{Values, http::Headers, mem::Instance}
};

pub mod form;

#[derive(Clone, Debug)]
pub struct Request {
    pub(crate) server: Instance<Server>,
    pub(crate) addr: SocketAddr,
    pub(crate) protocol: String,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) queries: Values,
    pub(crate) headers: Headers,
    pub(crate) host: String,
    pub(crate) cookies: Cookies,
    pub(crate) session: Session,
    pub(crate) body: Bytes,
    pub(crate) parameters: Values,
    pub(crate) form: Form, // TODO: need to make it pub(crate)
}

impl Into<serde_json::Value> for Request {
    fn into(self) -> serde_json::Value {
        serde_json::json!({
            "ip": &self.queries,
            "protocol": &self.protocol,
            "method": &self.method,
            "path": &self.path,
            "queries": &self.queries,
            "headers": &self.headers,
            "host": &self.host,
            "cookies": &self.cookies,
            "session": &self.session,
            "parameters": &self.parameters,
        })
    }
}

impl Request {
    #[inline]
    pub fn ip(&self) -> IpAddr {
        self
            .addr
            .ip()
    }

    #[inline]
    pub fn protocol(&self) -> String {
        self
            .protocol
            .clone()
    }

    #[inline]
    pub fn method(&self) -> String {
        self
            .path
            .clone()
    }

    #[inline]
    pub fn path(&self) -> String {
        self
            .path
            .clone()
    }

    #[inline]
    pub fn host(&self) -> String {
        self
            .path
            .clone()
    }

    pub fn is_json(&self) -> bool {
        self.header("content-type")
            .split(';')
            .next()
            .map(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
            .unwrap_or(false)
    }

    pub fn parameter(&self, key: impl Into<String>) -> String {
        self
            .parameters
            .get(&key.into())
            .unwrap_or(&String::new())
            .into()
    }

    pub fn query(&self, key: impl Into<String>) -> String {
        self
            .queries
            .get(&key.into())
            .unwrap_or(&String::new())
            .into()
    }

    pub fn query_default<T>(&self, key: impl Into<String>, default: T) -> T
    where
        T: std::str::FromStr
    {
        self.queries
            .get(&key.into())
            .and_then(|v| v.parse::<T>().ok())
            .unwrap_or(default)
    }

    #[inline]
    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn value(&self, key: impl Into<String>) -> String {
        self
            .form
            .values
            .get(&key.into())
            .unwrap_or(&String::new())
            .into()
    }

    pub fn file(&self, k: impl Into<String>) -> Option<&File> {
        self
            .form
            .files
            .get(&k.into())
    }

    #[inline]
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    #[inline]
    pub fn form(&self) -> &Form {
        &self.form
    }

    #[inline]
    pub fn parse_json<J: DeserializeOwned>(&self) -> Result<J, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    pub fn header(&self, name: &str) -> String {
        self
            .headers
            .get(&String::from(name))
            .unwrap_or(&String::new())
            .into()
    }

    pub fn cookie(&self, k: impl Into<String>) -> String {
        self
            .cookies
            .get(&k.into())
            .unwrap_or(&String::new())
            .into()
    }

    pub fn cookies(&self) -> &Cookies {
        &self.cookies
    }

    #[inline]
    pub fn server(&self) -> &mut Server {
        self
            .server
            .as_mut()
    }
}