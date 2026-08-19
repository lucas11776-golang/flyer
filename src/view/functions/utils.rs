use std::collections::HashMap;

use tera::{Tera, Value, to_value};

use crate::{utils, view::functions::HelperFunctions};


pub(crate) struct UtilsHelperFunctions;

impl HelperFunctions for UtilsHelperFunctions {
    fn register(engine: &mut Tera) {
        engine.register_function("env", Self::env());
        engine.register_function("url", Self::url());
    }
}

impl UtilsHelperFunctions {
    fn env() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let key = Self::get_arg(args, "name")
                .unwrap();

            to_value(utils::env::env(key)).map_err(|err| err.into())
        };
    }

    fn url() -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            let path = Self::get_arg(args, "path")
                .map(|v| String::from(v).trim_matches('/').into())
                .unwrap_or(String::new());

            to_value(crate::utils::url::url(&path)).map_err(|err| err.into())
        };
    }
}

