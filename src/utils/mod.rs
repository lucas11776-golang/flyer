use std::{collections::HashMap, mem::take};

pub mod url;

pub type Values = HashMap<String, String>;
pub type Configuration = HashMap<String, String>;