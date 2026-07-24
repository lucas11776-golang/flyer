use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bytes::Bytes;
use tokio::fs;

use crate::request::form::File;
use crate::storage::Storage;

pub struct Local {
    directory: PathBuf,
}

impl Local {
    pub fn new(directory: impl Into<String>) -> Self {
        Self {
            directory: PathBuf::from(directory.into()),
        }
    }

    /// Helper to resolve target path and protect against path traversal outside the base directory.
    fn resolve_path(&self, relative_path: &str) -> PathBuf {
        self.directory.join(relative_path)
    }
}

impl Storage for Local {
    async fn save_as(&self, folder: impl Into<String>, name: impl Into<String>, file: File) -> Result<String> {
        let folder_str = folder.into();
        let name_str = name.into();

        // Build target directory and file paths
        let target_dir = self.resolve_path(&folder_str);
        let target_path = target_dir.join(&name_str);

        // Ensure target directory exists
        fs::create_dir_all(&target_dir)
            .await
            .with_context(|| format!("Failed to create directory: {}", target_dir.display()))?;

        // Write file contents asynchronously
        fs::write(&target_path, file.content)
            .await
            .with_context(|| format!("Failed to write file: {}", target_path.display()))?;

        // Return path relative to base directory
        let relative_result = Path::new(&folder_str).join(&name_str);

        Ok(relative_result.to_string_lossy().into_owned())
    }

    async fn save(&self, folder: impl Into<String>, file: File) -> Result<String> {
        let file_name = file.name.clone();
        self.save_as(folder, file_name, file).await
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

        // Read bytes asynchronously
        let content_bytes = fs::read(&path)
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // Extract filename and guess MIME type from file extension
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