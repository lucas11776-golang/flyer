use std::collections::HashMap;

use anyhow::Result;
use bytes::Bytes;

use crate::{
    // storage::{DEFAULT_STORAGE, GLOBAL_STORAGE},
    // storage::{DEFAULT_STORAGE},
    utils::Values
};

pub type Files = HashMap<String, File>;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub mime: String,
    pub content: Bytes,
}

#[derive(Debug, Clone, Default)]
pub struct Form {
    pub values: Values,
    pub files: Files,
}

impl Form {
    pub fn new(values: Values, files: Files) -> Self {
        return Self {
            values: values,
            files: files,
        }
    }
}

impl File {
    pub fn new(name: &str, content: Bytes) -> Self {
        return Self {
            name: name.into(),
            mime: mime_guess::from_ext(&name.split(".").last().map(|v| String::from(v))
                .unwrap_or(String::new()))
                .first()
                .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM)
                .to_string(), 
            content: content,
        }
    }

    pub(crate) fn create(name: impl Into<String>, mime: impl Into<String>, content: Bytes) -> Self {
        return Self {
            name: name.into(),
            mime: mime.into(),
            content: content,
        }
    }

    #[allow(static_mut_refs)]
    pub async fn save_as(&self, folder: impl Into<String>, name: impl Into<String>,) -> Result<String> {
        todo!()
        // unsafe  {
        //     return GLOBAL_STORAGE
        //         .get(DEFAULT_STORAGE)
        //         .unwrap()
        //         .save_as(folder, name, self);
        // }
    }

    #[allow(static_mut_refs)]
    pub async fn save(&self, folder: impl Into<String>,) -> Result<String> {
        todo!()
        // unsafe  {
        //     return GLOBAL_STORAGE
        //         .get(DEFAULT_STORAGE)
        //         .unwrap()
        //         .save(folder, self);
        // }
    }
}
