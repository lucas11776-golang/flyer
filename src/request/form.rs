use std::{collections::HashMap, io::{Result}, path::{Path, PathBuf}};

use tokio::{fs, io::AsyncWriteExt};

use crate::utils::Values;

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
    pub fn new(name: &str, mime: &str, content: Vec<u8>) -> File {
        return Self {
            name: name.to_string(),
            mime: mime.to_string(),
            content: content,
        }
    }

    pub async fn save_as(&self, directory: &str, name: &str) -> Result<String> {
        let mut path = PathBuf::from(directory);

        fs::create_dir_all(&path).await?;

        path.push(name);

        let mut file = fs::File::create(&path).await?;
        file.write_all(&self.content).await?;

        return Ok(path.to_string_lossy().to_string());
    }

    pub async fn save(&self, directory: &str) -> Result<String> {
        let extension = Path::new(&self.name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext))
            .unwrap_or_default();

        let name = format!("{}{}", uuid::Uuid::new_v4().to_string(), extension);

        return self.save_as(directory, &name).await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    #[tokio::test]
    async fn test_save_as() {
        let content = b"hello world".to_vec();
        let file = File::new("test.txt", "text/plain", content.clone());
        let dir = "test_uploads";
        let name = "saved_test.txt";
        
        let path_str = file.save_as(dir, name).await.unwrap();
        let path = Path::new(&path_str);
        
        assert!(path.exists());
        assert_eq!(std_fs::read(path).unwrap(), content);
        
        // Cleanup
        let _ = std_fs::remove_file(path);
        let _ = std_fs::remove_dir(dir);
    }

    #[tokio::test]
    async fn test_save() {
        let content = b"hello world".to_vec();
        let file = File::new("test.jpg", "image/jpeg", content.clone());
        let dir = "test_uploads_save";
        
        let path_str = file.save(dir).await.unwrap();
        let path = Path::new(&path_str);
        
        assert!(path.exists());
        assert!(path_str.contains(dir));
        assert!(path_str.ends_with(".jpg"));
        assert_eq!(std_fs::read(path).unwrap(), content);
        
        // Cleanup
        let _ = std_fs::remove_file(path);
        let _ = std_fs::remove_dir(dir);
    }
}