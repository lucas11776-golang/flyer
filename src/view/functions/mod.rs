use tera::Tera;

use crate::view::functions;

pub(crate) mod utils;
pub(crate) mod session;

pub(crate) fn register<'r>(engine: &mut Tera) {
    register_session_functions(engine);
    register_utils_functions(engine);
}

pub(crate) fn register_session_functions(render: &mut Tera) {
    functions::session::register_global_functions(render);
}

pub(crate) fn register_utils_functions<'r>(engine: &mut Tera) {
    functions::utils::register(engine);
}