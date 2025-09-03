use std::{collections::HashMap};

use crate::utils::Values;


pub type Headers = HashMap<String, String>;
pub type Files = HashMap<String, File>;

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub content_type: String,
    pub content: Vec<u8>,
    pub size: usize,
}

#[derive(Debug)]
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