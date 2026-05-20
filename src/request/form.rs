use std::{collections::HashMap, path::Path};

use anyhow::Result;

use crate::{storage::{DEFAULT_STORAGE_NAME, GLOBAL_STORAGE}, utils::Values};

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
            name: name.to_string(),
            mime: mime.to_string(),
            content: content,
        }
    }

    #[allow(static_mut_refs)]
    pub async fn save_as(&self, folder: &str, name: &str) -> Result<String> {
        unsafe  {
            return GLOBAL_STORAGE.get(DEFAULT_STORAGE_NAME)
                .unwrap()
                .save_as(folder, name, self);
        }
    }

    pub async fn save(&self, folder: &str) -> Result<String> {
        let extension = Path::new(&self.name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{}", ext))
            .unwrap_or_default();

        return self.save_as(folder, &format!("{}.{}", uuid::Uuid::new_v4().to_string().replace("-", ""), extension)).await;
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