use std::{collections::HashMap};

pub mod url;

pub type Values = HashMap<String, String>;
pub type Configuration = HashMap<String, String>;

pub fn merge<T>(items: Vec<Vec<T>>) -> Vec<T> {
    let mut merged: Vec<T> = vec![];

    for item in items {
        merged.extend(item);
    }

    return merged
}
