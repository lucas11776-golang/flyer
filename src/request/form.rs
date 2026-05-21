use std::collections::HashMap;

use anyhow::Result;

use crate::{
    storage::{DEFAULT_STORAGE, GLOBAL_STORAGE},
    utils::Values
};

pub type Files = HashMap<String, File>;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub mime: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Form {
    pub values: Values,
    pub files: Files,
}

impl File {
    pub fn new(name: &str, content: Vec<u8>) -> Self {
        // TODO: use mime_guess::Mime in (mime)
        return Self {
            name: String::from(name),
            mime: String::new(), 
            content: content,
        }
    }

    pub(crate) fn create(name: &str, mime: &str, content: Vec<u8>) -> Self {
        return Self {
            name: String::from(name),
            mime: String::from(mime),
            content: content,
        }
    }

    #[allow(static_mut_refs)]
    pub async fn save_as(&self, folder: &str, name: &str) -> Result<String> {
        unsafe  {
            return GLOBAL_STORAGE
                .get(DEFAULT_STORAGE)
                .unwrap()
                .save_as(folder, name, self);
        }
    }

    #[allow(static_mut_refs)]
    pub async fn save(&self, folder: &str) -> Result<String> {
        unsafe  {
            return GLOBAL_STORAGE
                .get(DEFAULT_STORAGE)
                .unwrap()
                .save(folder, self);
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