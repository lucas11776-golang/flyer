use std::{collections::HashMap, io::Result, time::{SystemTime, UNIX_EPOCH}};

pub mod url;
pub mod string;
pub mod encrypt;
pub mod cookie;
pub mod logger;
pub mod server;
pub mod async_peek;

pub type Values = HashMap<String, String>;

pub type Headers = HashMap<String, String>;

pub type Configuration = HashMap<String, String>;

pub fn merge<T>(items: Vec<Vec<T>>) -> Vec<T> {
    let mut merged: Vec<T> = vec![];

    for item in items {
        merged.extend(item);
    }

    return merged
}

pub fn timestamp() -> Result<u128> {
    return Ok(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
}

pub fn load_env(path: &str) {
    dotenv::from_filename(path).unwrap();
}

pub fn env(name: &str) -> String {
    let var = dotenv::var(name);

    if var.is_err() {
        return String::new();
    }
    
    return var.unwrap();
}

