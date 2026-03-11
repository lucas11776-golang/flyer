use std::collections::HashMap;

use tera::{Value, to_value};

use crate::utils::{Values, env};

pub(crate) struct SessionFunctions;

impl SessionFunctions {
    pub fn session(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let session = values.get(args.get("name").unwrap().as_str().unwrap());

            if session.is_none() {
                return Ok(to_value("").unwrap());
            }

            return Ok(to_value(session.unwrap()).unwrap());
        };
    }

    pub fn session_has(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let session = values.get(args.get("name").unwrap().as_str().unwrap());

            if session.is_none() {
                return Ok(to_value(true).unwrap());
            }

            return Ok(to_value(false).unwrap());
        };
    }

    pub fn error_has(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let error = values.get(args.get("name").unwrap().as_str().unwrap());
            let class = args.get("class");

            if error.is_none() && class.is_none() {
                return Ok(to_value(false).unwrap());
            }

            if error.is_some() && class.is_none() {
                return Ok(to_value(true).unwrap());
            }

            if error.is_none() {
                return Ok(to_value("").unwrap());
            }

            return Ok(to_value(class.unwrap())?);
        };
    }

    pub fn error(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let session = values.get(args.get("name").unwrap().as_str().unwrap());

            if session.is_none() {
                return Ok(to_value("").unwrap());
            }

            return Ok(to_value(session.unwrap()).unwrap());
        };
    }

    pub fn old(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            return Ok(to_value(values.get(args.get("name").unwrap().as_str().unwrap()).or(Some(&String::new())).unwrap()).unwrap());
        };
    }

    pub fn env() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            return Ok(to_value(env(&args.get("name").unwrap().to_string())).unwrap());
        };
    }

    pub fn url() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let mut path = String::new();

            if let Some(p) = args.get("path") {
                path = p.as_str().unwrap().trim_start_matches("/").trim_end_matches("/").to_owned();
            }

            return Ok(to_value(format!("{}/{}", env("APP_URL").trim_end_matches("/"), path)).unwrap());
        };
    }
}