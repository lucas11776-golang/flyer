use std::path::Path;

use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::{Config};
use aws_sdk_s3::{primitives::ByteStream, Client};
use uuid::Uuid;

use crate::request::form::File;
use crate::storage::Storage;

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
    pub endpoint_url: Option<String>,
    pub session_token: Option<String>,
}

impl S3Config {
    pub fn new(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        region: impl Into<String>,
        bucket: impl Into<String>,
    ) -> Self {
        Self {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: region.into(),
            bucket: bucket.into(),
            endpoint_url: None,
            session_token: None,
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint_url = Some(endpoint.into());
        self
    }

    pub fn with_session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }
}

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub fn new(config: S3Config) -> Self {
        let credentials = Credentials::new(
            config.access_key,
            config.secret_key,
            config.session_token,
            None,
            "manual",
        );

        let mut builder = Config::builder()
            .behavior_version_latest()
            .region(Region::new(config.region))
            .credentials_provider(credentials);

        if let Some(endpoint) = config.endpoint_url {
            builder = builder.endpoint_url(endpoint);
        }

        let client = Client::from_conf(builder.build());

        Self {
            client,
            bucket: config.bucket,
        }
    }

    fn make_key(&self, folder: &str, name: &str) -> String {
        let folder = folder.trim_matches('/');
        let name = name.trim_start_matches('/');

        if folder.is_empty() {
            return name.to_string();
        }
        
        format!("{folder}/{name}")
    }

    pub fn generate_random_filename(&self, original_name: &str) -> String {
        let extension = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();

        format!("{}{}", Uuid::new_v4(), extension)
    }
}

impl Storage for S3Storage {
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
        let random_name = self.generate_random_filename(&file.name);
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