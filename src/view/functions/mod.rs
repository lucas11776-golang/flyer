use tera::Tera;

use crate::{request::Request, session::Session, view::functions};

pub(crate) mod utils;
pub(crate) mod session;

pub(crate) fn register<'r>(engine: &mut Tera, req: &Request) {
    register_session_functions(engine, &req.session);
    register_utils_functions(engine);
}

pub(crate) fn register_session_functions(render: &mut Tera, s: &Session) {
    functions::session::register(render, s);
}

pub(crate) fn register_utils_functions<'r>(engine: &mut Tera) {
    functions::utils::register(engine);
}
