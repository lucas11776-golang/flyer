use std::collections::HashMap;

use anyhow::Result;
use bytes::Bytes;

use crate::{
    storage::{DEFAULT_STORAGE, save, save_as},
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

impl Form {
    pub fn value(&self, k: impl Into<String>) -> String {
        self
            .values
            .get(&k.into())
            .map(|v| String::from(v))
            .unwrap_or(String::new())
    }

    pub fn file(&self, k: impl Into<String>) -> Option<File> {
        self
            .files
            .get(&k.into())
            .map(|f| f.clone())
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
        save_as(DEFAULT_STORAGE, folder, name, self.clone()).await
    }

    #[allow(static_mut_refs)]
    pub async fn save(&self, folder: impl Into<String>,) -> Result<String> {
        save(DEFAULT_STORAGE, folder, self.clone()).await
    }
}
