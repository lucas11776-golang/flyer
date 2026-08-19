use std::collections::HashMap;

use tera::Tera;

use tera::{Value, to_value};

use crate::{
    cookies::Cookies,
    session::Session,
    utils::{Values, http::Headers},
    view::functions::{
        request::RequestHelperFunctions,
        utils::UtilsHelperFunctions
    }
};

pub(crate) mod utils;
pub(crate) mod request;

tokio::task_local! {
    pub(crate) static VIEW_REQUEST_DATA: ViewRequestData;
}

pub trait HelperFunctions {
    fn register(engine: &mut Tera);

    fn get_arg<'a>(args: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        args.get(key).and_then(|v| v.as_str())
    }
}

pub(crate) struct ViewRequestData {
    pub(crate) queries: Values,
    pub(crate) parameters: Values,
    pub(crate) headers: Headers,
    pub(crate) cookies: Cookies,
    pub(crate) session: Session,
}

// TODO: cleanup
pub(crate) fn register<'r>(engine: &mut Tera) {
    register_session_functions(engine);
    register_utils_functions(engine);
}

pub(crate) fn register_session_functions(render: &mut Tera) {
    RequestHelperFunctions::register(render);
}

pub(crate) fn register_utils_functions<'r>(engine: &mut Tera) {
    UtilsHelperFunctions::register(engine);
}