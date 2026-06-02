use std::collections::HashMap;

use anyhow::Result;

use crate::{request::form::File, storage::local::LocalStorage};

pub mod local;

pub(crate) static mut GLOBAL_STORAGE: std::sync::LazyLock<HashMap<String, Box<dyn Storage>>> = std::sync::LazyLock::new(|| {
    let mut storages = HashMap::new();

    storages.insert(String::from(DEFAULT_STORAGE), Box::new(LocalStorage::new(None)) as Box<dyn Storage>);

    return storages;
});

pub const DEFAULT_STORAGE: &'static str = "default";

#[allow(async_fn_in_trait)]
pub trait Storage {
    fn save_as(&self, folder: &str, name: &str, file: &File) -> Result<String>;
    fn save(&self, folder: &str, name: &File) -> Result<String>;
    fn delete(&self, filename: &str) -> Result<()>;
    fn exits(&self, filename: &str) -> Result<bool>;
    fn get(&self, filename: &str) -> Result<File>;
}

#[allow(static_mut_refs)]
pub fn add(name: &str, storage: Box<impl Storage + 'static>) {
    unsafe  {
        GLOBAL_STORAGE.insert(String::from(name), storage);
    }
}

#[allow(static_mut_refs)]
pub fn save_as(storage: &str, folder: &str, name: &str, file: &File) -> Result<String> {
    unsafe {
        return GLOBAL_STORAGE.get(storage)
            .unwrap()
            .save_as(folder, name, file);
    }
}

#[allow(static_mut_refs)]
pub fn save(storage: &str, folder: &str, file: &File) -> Result<String> {
    unsafe {
        return GLOBAL_STORAGE.get(storage)
            .unwrap()
            .save(folder, file);
    }
}

#[allow(static_mut_refs)]
pub fn delete(storage: &str, path: &str) -> Result<()> {
    unsafe {
        return GLOBAL_STORAGE.get(storage)
            .unwrap()
            .delete(path);
    }
}

#[allow(static_mut_refs)]
pub fn exists(storage: &str, path: &str) -> Result<bool> {
    unsafe {
        return GLOBAL_STORAGE.get(storage)
            .unwrap()
            .exits(path);
    }
}


#[allow(static_mut_refs)]
pub fn get(storage: &str, path: &str) -> Result<File> {
    unsafe {
        return GLOBAL_STORAGE.get(storage)
            .unwrap()
            .get(path);
    }
}