use std::ops::Add;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bytes::Bytes;
use tokio::fs;
use uuid::Uuid;

use crate::request::form::File;
use crate::storage::Storage;

pub struct LocalStorage {
    directory: PathBuf,
}

impl LocalStorage {
    pub fn new(directory: impl Into<String>) -> Self {
        Self {
            directory: PathBuf::from(directory.into()),
        }
    }

    fn resolve_path(&self, relative_path: &str) -> PathBuf {
        self.directory.join(relative_path)
    }
}

impl Storage for LocalStorage {
    async fn save_as(&self, folder: impl Into<String>, name: impl Into<String>, file: File) -> Result<String> {
        let folder_str = folder.into();
        let name_str = format!("{}.{}", name.into(), file.name.split(".").last().unwrap());

        let target_dir = self.resolve_path(&folder_str);
        let target_path = target_dir.join(&name_str);

        fs::create_dir_all(&target_dir)
            .await
            .with_context(|| format!("Failed to create directory: {}", target_dir.display()))?;

        fs::write(&target_path, file.content)
            .await
            .with_context(|| format!("Failed to write file: {}", target_path.display()))?;

        let relative_result = Path::new(&folder_str).join(&name_str);

        Ok(format!("{}/{}", self.directory.to_string_lossy(), relative_result.to_string_lossy().into_owned()))
    }

    async fn save(&self, folder: impl Into<String>, file: File) -> Result<String> {
        let mut filename = Uuid::new_v4().to_string().replace("-", "");
        let extension = file.name.split(".").collect::<Vec<&str>>();

        if extension.len() > 1 {
            filename = filename.add(&format!(".{}", extension.last().unwrap()));
        }

        self.save_as(folder, filename, file).await
    }

    async fn delete(&self, filename: impl Into<String>) -> Result<()> {
        let path = self.resolve_path(&filename.into());

        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path)
                .await
                .with_context(|| format!("Failed to delete file: {}", path.display()))?;
        }

        Ok(())
    }

    async fn exists(&self, filename: impl Into<String>) -> Result<bool> {
        let path = self.resolve_path(&filename.into());
        Ok(fs::try_exists(&path).await.unwrap_or(false))
    }

    async fn get(&self, filename: impl Into<String>) -> Result<File> {
        let filename_str = filename.into();
        let path = self.resolve_path(&filename_str);

        let content_bytes = fs::read(&path)
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let file_name = Path::new(&filename_str)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| filename_str.clone());

        let mime = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();

        Ok(File {
            name: file_name,
            mime,
            content: Bytes::from(content_bytes),
        })
    }
}