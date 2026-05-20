use std::collections::HashMap;

use anyhow::Result;

use crate::{request::form::File, storage::local::LocalStorage};

pub mod local;

pub(crate) static mut GLOBAL_STORAGE: std::sync::LazyLock<HashMap<String, Box<dyn Storage>>> = std::sync::LazyLock::new(|| {
    let mut storages = HashMap::new();

    storages.insert(String::from(DEFAULT_STORAGE_NAME), Box::new(LocalStorage::new(None)) as Box<dyn Storage>);

    return storages;
});

pub(crate) const DEFAULT_STORAGE_NAME: &'static str = "public";

#[allow(async_fn_in_trait)]
pub trait Storage {
    fn save_as(&self, folder: &str, name: &str, file: &File) -> Result<String>;
    fn save(&self, folder: &str, name: &str) -> Result<String>;
    fn delete(&self, filename: &str) -> Result<()>;
    fn exits(&self, filename: &str) -> Result<bool>;
}


#[allow(static_mut_refs)]
pub fn add(name: &str, storage: Box<impl Storage + 'static>) {
    unsafe  {
        GLOBAL_STORAGE.insert(String::from(name), storage);
    }
}