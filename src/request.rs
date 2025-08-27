use std::{collections::HashMap, io::{Error, Result}};
use urlencoding::decode;


pub type Headers = HashMap<String, String>;
pub type Values = HashMap<String, String>;
pub type Files = HashMap<String, File>;


#[derive(Debug)]
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

#[derive(Debug)]
pub struct Request {
    pub(crate) ip: String,
    pub host: String,
    pub method: String,
    pub path: String,
    pub parameters: Values,
    pub protocol: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub values: Values,
    pub files: Files,
}

struct MultipartField {
    content_disposition: String,
    name: String,
    filename: String,
    content_type: String,
    content: String,
}

impl Request {
    pub fn header(&self, key: &str) -> String {
        return self.headers.get(key).get_or_insert(&"".to_string()).to_string()
    }

    pub fn parameter(&self, key: &str) -> String {
        return self.parameters.get(key).get_or_insert(&"".to_string()).to_string()
    }

    pub fn value(&self, key: &str) -> Option<String> {
        return Some(self.values.get(key).get_or_insert(&"".to_owned()).to_string());
    }

    pub fn file(&self, key: &str) -> Option<&File> {
        return self.files.get(key);
    }

    pub fn ip(&self) -> String {
        return self.ip.to_owned();
    }
}