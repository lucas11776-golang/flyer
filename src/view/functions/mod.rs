use tera::Tera;

use crate::{request::Request, session::Session, view::functions};

pub(crate) mod utils;
pub(crate) mod session;

pub(crate) fn register<'r>(engine: &'r mut Tera, req: &'r mut Request) {
    req.session.as_mut().map(|session| register_session_functions(engine, session));
    register_utils_functions(engine);
}

pub(crate) fn register_session_functions<'r>(render: &'r mut Tera, s: &mut Box<dyn Session + 'static>) {
    functions::session::register(render, s);
}

pub(crate) fn register_utils_functions<'r>(engine: &'r mut Tera) {
    functions::utils::register(engine);
}
