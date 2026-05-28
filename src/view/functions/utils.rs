use std::collections::HashMap;

use tera::{Tera, Value, to_value};

pub(crate) fn register<'r>(engine: &'r mut Tera) {
    engine.register_function("env", env());
    engine.register_function("url", url());
}

fn env() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        return Ok(to_value(crate::utils::env(args.get("name").unwrap().as_str().unwrap())).unwrap());
    };
}

// TODO: refactor.
fn url() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        let mut path = String::new();

        if let Some(p) = args.get("path") {
            path = p.as_str().unwrap().trim_start_matches("/").trim_end_matches("/").to_owned();
        }

        return Ok(to_value(crate::utils::url::url(&path)).unwrap());
    };
}