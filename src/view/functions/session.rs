use std::collections::HashMap;
use tera::{to_value, Tera, Value};

use crate::{session::Session, utils::Values};

#[derive(Clone, Debug, Default)]
pub struct SessionState {
    pub session: Values,
    pub errors: Values,
    pub olds: Values,
    pub flashes: Values,
}

impl SessionState {
    pub fn from_session(s: &Session) -> Self {
        Self {
            session: s.session(),
            errors: s.errors(),
            olds: s.olds(),
            flashes: s.flashes(),
        }
    }
}

tokio::task_local! {
    pub static CURRENT_SESSION: SessionState;
}

pub(crate) fn register_global_functions(render: &mut Tera) {
    render.register_function("session", session_fn());
    render.register_function("session_has", session_has_fn());
    render.register_function("error", error_fn());
    render.register_function("error_has", error_has_fn());
    render.register_function("old", old_fn());
    render.register_function("flash", flash_fn());
    render.register_function("flash_has", flash_has_fn());
}

fn get_arg<'a>(args: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

fn session_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        CURRENT_SESSION.try_with(|s| {
            match s.session.get(name) {
                Some(val) => to_value(val),
                None => to_value(""),
            }
        })
        .unwrap()
        .map_err(|err| err.into())
    }
}

fn session_has_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        CURRENT_SESSION.try_with(|s| {
            to_value(s.session.get(name).is_some())
        })
        .unwrap()
        .map_err(|err| err.into())
    }
}

fn error_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        CURRENT_SESSION.try_with(|s| {
            match s.errors.get(name) {
                Some(err) => to_value(err),
                None => to_value(""),
            }
        })
        .unwrap()
        .map_err(|err| err.into())
    }
}

fn error_has_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        let class = args.get("class");

        CURRENT_SESSION.try_with(|s| {
            let error = s.errors.get(name);

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

fn old_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        CURRENT_SESSION.try_with(|s| {
            match s.olds.get(name) {
                Some(val) => to_value(val),
                None => to_value(""),
            }
        })
        .unwrap()
        .map_err(|err| err.into())
    }
}

fn flash_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        CURRENT_SESSION.try_with(|s| {
            match s.flashes.get(name) {
                Some(val) => to_value(val),
                None => to_value(""),
            }
        })
        .unwrap()
        .map_err(|err| err.into())
    }
}

fn flash_has_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        CURRENT_SESSION.try_with(|s| {
            to_value(s.flashes.get(name).is_some())
        })
        .unwrap()
        .map_err(|err| err.into())
    }
}