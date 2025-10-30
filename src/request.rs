use std::{collections::HashMap};

use crate::utils::Values;

pub type Headers = HashMap<String, String>;
pub type Files = HashMap<String, File>;

pub struct File {
    pub name: String,
    pub content_type: String,
    pub content: Vec<u8>,
    pub size: usize,
}

pub struct MultipartForm {
    pub values: Values,
    pub files: Files,
}

pub struct Request {
    pub ip: String,
    pub host: String,
    pub method: String,
    pub path: String,
    pub query: Values,
    pub parameters: Values,
    pub protocol: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub values: Values,
    pub files: Files,
}

impl Request {
    pub fn new(method: &str, path: &str, headers: Values, body: Vec<u8>) -> Self {
        return Self {
            ip: "".to_owned(),
            host: "".to_owned(),
            method: method.to_owned(),
            path: path.to_owned(),
            query: Values::new(),
            parameters: Values::new(),
            protocol: "HTTP/1.1".to_string(),
            headers: headers,
            body: body,
            values: Values::new(),
            files: Files::new(),
        }
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

    pub fn value(&self, key: &str) -> Option<String> {
        return Some(self.values.get(key).get_or_insert(&"".to_owned()).to_string());
    }

    pub fn file(&self, key: &str) -> Option<&File> {
        return self.files.get(key);
    }
}