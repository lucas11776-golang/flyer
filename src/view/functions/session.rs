use std::collections::HashMap;
use tera::{to_value, Tera, Value};

use crate::{session::Session, view::functions::VIEW_REQUEST_DATA};

// tokio::task_local! {
//     pub(crate) static VIEW_REQUEST_DATA: Session;
// }

pub(crate) fn register_global_functions(render: &mut Tera) {
    render.register_function("session", session_fn());
    render.register_function("session_has", session_has_fn());
    render.register_function("errors", errors_fn());
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
        VIEW_REQUEST_DATA
            .try_with(|data| to_value(data.session.get(name)))
            .unwrap()
            .map_err(|err| err.into())
    }
}

fn session_has_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        VIEW_REQUEST_DATA
            .try_with(|s| to_value(!s.session.get(name).is_empty()))
            .unwrap()
            .map_err(|err| err.into())
    }
}

fn error_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        VIEW_REQUEST_DATA
            .try_with(|s|  to_value(s.session.error(name)))
            .unwrap()
            .map_err(|err| err.into())
    }
}

fn errors_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |_| {
        VIEW_REQUEST_DATA
            .try_with(|s| to_value(s.session.errors.clone()))
            .unwrap()
            .map_err(|err| err.into())
    }
}

fn error_has_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
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

fn old_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        VIEW_REQUEST_DATA
            .try_with(|s| to_value(s.session.old(name)))
            .unwrap()
            .map_err(|err| err.into())
    }
}

fn flash_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        VIEW_REQUEST_DATA
            .try_with(|s| to_value( s.session.flash(name)))
            .unwrap()
            .map_err(|err| err.into())
    }
}

fn flash_has_fn() -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> + Send + Sync + 'static {
    |args| {
        let name = get_arg(args, "name").unwrap_or_default();
        VIEW_REQUEST_DATA
            .try_with(|s|   to_value(!s.session.flash(name).is_empty()))
            .unwrap()
            .map_err(|err| err.into())
    }
}