use async_std::task::block_on;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use std::path::PathBuf;

use crate::{request::form::File, storage::Storage};

const STORAGE_DIRECTORY: &'static str = "storage";

pub struct LocalStorage {
    directory: String
}

impl LocalStorage {
    pub fn new(directory: Option<&str>) -> Self {
        return Self {
            directory: directory.map(|v| String::from(v)).unwrap_or(String::from(STORAGE_DIRECTORY))
        }
    }

    fn extension(&self, name: String) -> String {
        return name.split(".")
            .last()
            .map(|v| format!(".{}", v))
            .unwrap_or(String::new());
    }
}

impl Storage for LocalStorage {
    fn save_as(&self, folder: &str, name: &str, file: &crate::request::form::File) -> anyhow::Result<String> {
        let folder_path = PathBuf::from(&self.directory).join(folder);
        
        block_on(tokio::fs::create_dir_all(&folder_path))?;

        let file_path = folder_path.join(name);
        let mut file_handle = block_on(tokio::fs::File::create(&file_path))?;

        block_on(file_handle.write_all(&file.content))?;

        return Ok(file_path.to_string_lossy().into_owned());
    }

    fn save(&self, folder: &str, file: &crate::request::form::File) -> anyhow::Result<String> {
        return self.save_as(
            folder,
            &format!("{}{}", String::from(Uuid::new_v4()).replace("-", ""), self.extension(file.name.clone())), 
            file
        );
    }

    fn delete(&self, path: &str) -> anyhow::Result<()> {
        block_on(tokio::fs::remove_file(path))?;

        return Ok(())
    }

    fn exits(&self, path: &str) -> anyhow::Result<bool> {
        return Ok(block_on(tokio::fs::try_exists(path))?)
    }
    
    fn get(&self, path: &str) -> anyhow::Result<crate::request::form::File> {
        // TODO: or split \ but need to check on windows
        let filename = path.split("/")
            .last()
            .map(|v| String::from(v))
            .unwrap_or(String::from(path));

        return Ok(File::new(&filename, std::fs::read(path).unwrap()));

    }
}