use std::collections::HashMap;

use tera::{Tera, Value, to_value};

use crate::request::Request;

pub(crate) fn register<'r>(engine: &'r mut Tera, _req: &'r mut Request) {
    engine.register_function("env", env());
    engine.register_function("url", url());
}

fn env() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        return Ok(to_value(crate::utils::env(&args.get("name").unwrap().to_string())).unwrap());
    };
}

// TODO: refactor.
fn url() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        let mut path = String::new();

        if let Some(p) = args.get("path") {
            path = p.as_str().unwrap().trim_start_matches("/").trim_end_matches("/").to_owned();
        }

        return Ok(to_value(format!("{}/{}", crate::utils::env("APP_URL").trim_end_matches("/"), path)).unwrap());
    };
}