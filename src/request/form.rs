use std::{collections::HashMap};

use crate::utils::Values;

pub type Files = HashMap<String, File>;

pub struct File {
    pub name: String,
    pub mime: String,
    pub content: Vec<u8>,
}

pub struct Form {
    pub values: Values,
    pub files: Files,
}

impl File {
    pub fn new(name: &str, mime: &str, content: Vec<u8>) -> File {
        return Self {
            name: name.to_string(),
            mime: mime.to_string(),
            content: content,
        }
    }
}

impl Form {
    pub fn new(values: Values, files: Files) -> Self {
        return Self {
            values: values,
            files: files
        }
    } 
}