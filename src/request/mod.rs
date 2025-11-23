pub mod form;
pub mod parser;

use std::cmp::Ordering;

use crate::{
    cookie::Cookies,
    request::form::{File, Files, Form},
    session::Session,
    utils::{Headers, Values}
};

pub struct Request {
    pub(crate) cookies: Box<Cookies>,
    pub(crate) session: Option<Box<dyn Session>>,
    pub ip: String,
    pub host: String,
    pub method: String,
    pub path: String,
    pub query: Values,
    pub parameters: Values,
    pub protocol: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub form: Form,
}

impl Request {
    pub fn new(method: &str, path: &str, headers: Values, body: Vec<u8>) -> Self {
        return Self {
            session: None,
            ip: "".to_owned(),
            host: "".to_owned(),
            method: method.to_owned(),
            path: path.to_owned(),
            query: Values::new(),
            parameters: Values::new(),
            protocol: "HTTP/1.1".to_string(),
            headers: headers,
            body: body,
            form: Form::new(Values::new(), Files::new()),
            cookies: Box::new(Cookies::new(Values::new())),
        }
    }

    pub(crate) fn is_asset(&mut self) -> bool {
        let end = self.path.split("/").last();

        if end.is_none() {
            return false;
        }

        let file_split: Vec<&str> = end.unwrap().split(".").collect();

        return file_split.len() > 1;
    }

    pub fn ip(&self) -> String {
        return self.ip.to_owned();
    }

    pub fn header(&self, key: &str) -> String {
        return self.headers.get(key).get_or_insert(&"".to_string()).to_string()
    }
    
    pub fn parameter(&self, key: &str) -> String {
        return self.parameters.get(key).get_or_insert(&"".to_string()).to_string()
    }

    pub fn query(&self, key: &str) -> String {
        return self.query.get(key).get_or_insert(&"".to_string()).to_string()
    }

    pub fn value(&self, key: &str) -> String {
        return self.form.values.get(key).get_or_insert(&"".to_owned()).to_string();
    }

    pub fn file(&self, key: &str) -> Option<&File> {
        return self.form.files.get(key);
    }

    pub fn content_type(&self) -> String {
        let content_type = self.header("content-type");
        let content_type_piece: Vec<&str> = content_type.split(";").collect();

        return content_type_piece.get(0).unwrap().to_string();
    }

    pub fn is_json(&self) -> bool {
        let header = self.header("content-type");
        let header_piece: Vec<&str> = header.split(";").collect();

        return  header_piece.get(0).unwrap().cmp(&"application/json") == Ordering::Equal;
    }

    pub fn session(&mut self) -> &mut Box<dyn Session + 'static> {
        return self.session.as_mut().unwrap();
    }

    pub fn cookies(&mut self) -> &mut Cookies {
        return &mut self.cookies;
    }
}