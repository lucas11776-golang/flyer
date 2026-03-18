use tera::Tera;

use crate::{
    request::Request,
    view::functions::{
        session::register as register_session_functions,
        utils::register as register_utils_functions,
    }
};

pub(crate) mod utils;
pub(crate) mod session;

pub(crate) fn register<'r>(engine: &'r mut Tera, req: &'r mut Request) {
    req.session.as_mut().map(|session| register_session_functions(engine, session));
    register_utils_functions(engine, req);
}