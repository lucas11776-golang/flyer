

use std::path::Path;

use anyhow::{Context, Result};
use aws_sdk_s3::{primitives::ByteStream, Client};
use uuid::Uuid;

use crate::request::form::File;
use crate::storage::Storage;

pub struct S3 {
    client: Client,
    bucket: String,
}

impl S3 {
    pub fn new(client: Client, bucket: impl Into<String>) -> Self {
        Self {
            client,
            bucket: bucket.into(),
        }
    }

    fn make_key(&self, folder: &str, name: &str) -> String {
        let folder = folder.trim_matches('/');
        let name = name.trim_start_matches('/');

        if folder.is_empty() {
            name.to_string()
        } else {
            format!("{folder}/{name}")
        }
    }
}


pub fn generate_random_filename(original_name: &str) -> String {
    let extension = Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    format!("{}{}", Uuid::new_v4(), extension)
}

impl Storage for S3 {
    async fn save_as(&self, folder: impl Into<String>, name: impl Into<String>, file: File) -> Result<String> {
        let key = self.make_key(&folder.into(), &name.into());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(file.content))
            .content_type(file.mime)
            .send()
            .await
            .with_context(|| format!("Failed to upload object to S3 at key: '{key}'"))?;

        Ok(key)
    }

    async fn save(&self, folder: impl Into<String>, file: File) -> Result<String> {
        let random_name = generate_random_filename(&file.name);
        self.save_as(folder, random_name, file).await
    }

    async fn delete(&self, filename: impl Into<String>) -> Result<()> {
        let key = filename.into();

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .with_context(|| format!("Failed to delete object from S3 at key: '{key}'"))?;

        Ok(())
    }

    async fn exists(&self, filename: impl Into<String>) -> Result<bool> {
        let key = filename.into();

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                // Check if the error represents a 404 Not Found response
                if let Some(service_err) = err.as_service_error() {
                    if service_err.is_not_found() {
                        return Ok(false);
                    }
                }
                Err(err).with_context(|| format!("Failed to check existence for S3 key: '{key}'"))
            }
        }
    }

    async fn get(&self, filename: impl Into<String>) -> Result<File> {
        let key = filename.into();

        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .with_context(|| format!("Failed to retrieve S3 object at key: '{key}'"))?;

        let mime = output
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let file_name = Path::new(&key)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| key.clone());

        let data = output
            .body
            .collect()
            .await
            .with_context(|| format!("Failed to read stream for S3 key: '{key}'"))?;

        Ok(File {
            name: file_name,
            mime,
            content: data.into_bytes(),
        })
    }
}