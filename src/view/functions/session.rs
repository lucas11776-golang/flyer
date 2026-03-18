use std::collections::HashMap;

use tera::{Tera, Value, to_value};

use crate::{session::Session, utils::Values};

pub(crate) fn register<'r>(render: &'r mut Tera, s: &mut Box<dyn Session + 'static>) {
    render.register_function("session", session(s.values()));
    render.register_function("session_has", session_has(s.values()));
    render.register_function("error_has", error_has(s.errors()));
    render.register_function("error", error(s.errors()));
    render.register_function("old", old(s.old_values()));
}

fn session(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        return Ok(match values.get(args.get("name").unwrap().as_str().unwrap()) {
            Some(value) => to_value(value).unwrap(),
            None => to_value(String::new()).unwrap(),
        });
    };
}

fn session_has(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        return Ok(match values.get(args.get("name").unwrap().as_str().unwrap()) {
            Some(_) => to_value(true).unwrap(),
            None => to_value(false).unwrap(),
        });
    };
}

fn error(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        return Ok(match values.get(args.get("name").unwrap().as_str().unwrap()) {
            Some(error) => to_value(error).unwrap(),
            None => to_value("").unwrap(),
        });
    };
}

fn error_has(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
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

fn old(values: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
    return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
        return Ok(match values.get(args.get("name").unwrap().as_str().unwrap()) {
            Some(error) => to_value(error).unwrap(),
            None => to_value(String::new()).unwrap(),
        });
    };
}