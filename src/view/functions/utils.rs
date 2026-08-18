use std::collections::HashMap;

use tera::{Tera, Value, to_value};

use crate::utils;

pub(crate) fn register<'r>(engine: &'r mut Tera) {
    engine.register_function("env", env());
    engine.register_function("url", url());
}

fn env() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        let key = args
            .get("name")
            .unwrap()
            .as_str()
            .unwrap();

        Ok(to_value(utils::env::env(key)).unwrap())
    };
}

// TODO: refactor.
fn url() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        let mut path = String::new();

        if let Some(p) = args.get("path") {
            path = p
                .as_str()
                .unwrap()
                .trim_start_matches("/")
                .trim_end_matches("/")
                .to_owned();
        }

        Ok(to_value(crate::utils::url::url(&path)).unwrap())
    };
}