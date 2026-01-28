use std::{collections::HashMap, io::{Result}};

use tokio::{io::AsyncWriteExt};

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

    pub async fn upload_as(&self, directory: &str, name: &str) -> Result<String> {
        let mut extension = String::new();
        let raw: Vec<&str> = self.name.split(".").collect();

        if let Some(ext) = raw.last() {
            extension = format!(".{}", ext);
        }

        let path = match directory.trim_end_matches("/").starts_with("/") {
            true => format!("{}/{}{}", directory.trim_end_matches("/"), name, extension),
            false => format!("{}{}", name, extension),
        };

        tokio::fs::File::create(path.clone())
            .await
            .unwrap()
            .write_all(&self.content)
            .await
            .unwrap();

        return Ok(path);
    }

    pub async fn upload(&self, directory: &str) -> Result<String> {
        return self.upload_as(directory, "jdaklsjnfhalkshfaskl").await;
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