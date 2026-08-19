use std::collections::HashMap;
use tera::{to_value, Tera, Value};

use crate::view::functions::{HelperFunctions, VIEW_REQUEST_DATA};

pub(crate) struct RequestHelperFunctions;

impl HelperFunctions for RequestHelperFunctions {
    fn register(engine: &mut Tera) {
        // QUERIES
        engine.register_function("query", Self::query());
        // PARAMETERS
        engine.register_function("parameter", Self::parameter());
        // HEADERS
        engine.register_function("header", Self::header());
        // COOKIES
        engine.register_function("cookie", Self::cookie());
        // SESSION
        engine.register_function("session", Self::session());
        engine.register_function("session_has", Self::session_has());
        engine.register_function("errors", Self::errors());
        engine.register_function("error", Self::error());
        engine.register_function("error_has", Self::error_has());
        engine.register_function("old", Self::old());
        engine.register_function("flash", Self::flash_fn());
        engine.register_function("flash_has", Self::flash_has());
    }
}

impl RequestHelperFunctions {
    // QUERIES
    fn query() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|data| to_value(data.queries.get(name).unwrap_or(&String::new())))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    // PARAMETERS
    fn parameter() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|data| to_value(data.parameters.get(name).unwrap_or(&String::new())))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    // HEADERS
    fn header() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|data| to_value(data.headers.get(name).unwrap_or(&String::new())))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    // HEADERS
    fn cookie() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|data| to_value(data.cookies.get(name).unwrap_or(&String::new())))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    // SESSION
    fn session() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|data| to_value(data.session.get(name)))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    fn session_has() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|s| to_value(!s.session.get(name).is_empty()))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    fn error() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|s|  to_value(s.session.error(name)))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    fn errors() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |_| {
            VIEW_REQUEST_DATA
                .try_with(|s| to_value(s.session.errors.clone()))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    fn error_has() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            let class = args.get("class");

            VIEW_REQUEST_DATA.try_with(|s| {
                // TODO: refactor to simplify
                let error = s.session.errors.get(name);

                if error.is_none() && class.is_none() {
                    return to_value(false);
                }
                if error.is_some() && class.is_none() {
                    return to_value(true);
                }
                if error.is_none() {
                    return to_value("");
                }
                to_value(class.unwrap())
            })
            .unwrap()
            .map_err(|err| err.into())
        }
    }

    fn old() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|s| to_value(s.session.old(name)))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    fn flash_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|s| to_value( s.session.flash(name)))
                .unwrap()
                .map_err(|err| err.into())
        }
    }

    fn flash_has() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
        |args| {
            let name = Self::get_arg(args, "name").unwrap_or_default();
            VIEW_REQUEST_DATA
                .try_with(|s|   to_value(!s.session.flash(name).is_empty()))
                .unwrap()
                .map_err(|err| err.into())
        }
    }
}