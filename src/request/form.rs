use std::collections::HashMap;

use crate::utils::Values;

pub type Files = HashMap<String, File>;

pub struct File {
    pub name: String,
    pub content_type: String,
    pub content: Vec<u8>,
    pub size: usize,
}

pub struct Form {
    pub values: Values,
    pub files: Files,
}

impl Form {
    pub(crate) fn new() -> Self {
        return Self {
            values: Values::new(),
            files: Files::new()
        }
    } 
}