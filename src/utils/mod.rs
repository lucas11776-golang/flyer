use std::collections::HashMap;

pub mod development;
pub mod future;
pub mod collections;
pub mod http;
pub mod url;
pub mod vec;
pub mod env;
pub mod server;
pub(crate) mod route;
pub mod mem;

pub type Values = HashMap<String, String>;