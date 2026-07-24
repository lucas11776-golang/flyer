use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use anyhow::{anyhow, Result};
use futures::future::BoxFuture;

use crate::{request::form::File, utils::future::SendFuture};

pub mod local;

pub mod aws;

static GLOBAL_STORAGE: LazyLock<RwLock<HashMap<String, Arc<dyn StorageErasure>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[allow(async_fn_in_trait)]
pub trait Storage: Send + Sync {
    async fn save_as(&self, folder: impl Into<String>, name: impl Into<String>, file: File) -> Result<String>;
    async fn save(&self, folder: impl Into<String>, file: File) -> Result<String>;
    async fn delete(&self, filename: impl Into<String>) -> Result<()>;
    async fn exists(&self, filename: impl Into<String>) -> Result<bool>;
    async fn get(&self, filename: impl Into<String>) -> Result<File>;
}

trait StorageErasure: Send + Sync {
    fn save_as<'a>(&'a self, folder: String, name: String, file: File) -> BoxFuture<'a, Result<String>>;
    fn save<'a>(&'a self, folder: String, file: File) -> BoxFuture<'a, Result<String>>;
    fn delete<'a>(&'a self, filename: String) -> BoxFuture<'a, Result<()>>;
    fn exists<'a>(&'a self, filename: String) -> BoxFuture<'a, Result<bool>>;
    fn get<'a>(&'a self, filename: String) -> BoxFuture<'a, Result<File>>;
}

impl<T: Storage + 'static> StorageErasure for T {
    fn save_as<'a>(&'a self, folder: String, name: String, file: File) -> BoxFuture<'a, Result<String>> {
        Box::pin(SendFuture(Storage::save_as(self, folder, name, file)))
    }

    fn save<'a>(&'a self, folder: String, file: File) -> BoxFuture<'a, Result<String>> {
        Box::pin(SendFuture(Storage::save(self, folder, file)))
    }

    fn delete<'a>(&'a self, filename: String) -> BoxFuture<'a, Result<()>> {
        Box::pin(SendFuture(Storage::delete(self, filename)))
    }

    fn exists<'a>(&'a self, filename: String) -> BoxFuture<'a, Result<bool>> {
        Box::pin(SendFuture(Storage::exists(self, filename)))
    }

    fn get<'a>(&'a self, filename: String) -> BoxFuture<'a, Result<File>> {
        Box::pin(SendFuture(Storage::get(self, filename)))
    }
}

pub fn add(name: impl Into<String>, storage: impl Storage + 'static) {
    GLOBAL_STORAGE
        .write()
        .expect("Storage registry lock poisoned")
        .insert(name.into(), Arc::new(storage));
}

fn get_storage(name: &str) -> Result<Arc<dyn StorageErasure>> {
    GLOBAL_STORAGE
        .read()
        .expect("Storage registry lock poisoned")
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow!("Storage instance '{name}' not found"))
}

pub async fn save_as(storage: &str, folder: impl Into<String>, name: impl Into<String>, file: File) -> Result<String> {
    get_storage(storage)?.save_as(folder.into(), name.into(), file).await
}

pub async fn save(storage: &str, folder: impl Into<String>, file: File) -> Result<String> {
    get_storage(storage)?.save(folder.into(), file).await
}

pub async fn delete(storage: &str, filename: impl Into<String>) -> Result<()> {
    get_storage(storage)?.delete(filename.into()).await
}

pub async fn exists(storage: &str, filename: impl Into<String>) -> Result<bool> {
    get_storage(storage)?.exists(filename.into()).await
}

pub async fn get(storage: &str, filename: impl Into<String>) -> Result<File> {
    get_storage(storage)?.get(filename.into()).await
}