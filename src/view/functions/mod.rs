use tera::Tera;

use crate::{
    cookies::Cookies,
    session::Session,
    utils::{Values, http::Headers},
    view::functions
};

pub(crate) mod utils;
pub(crate) mod session;

tokio::task_local! {
    pub(crate) static VIEW_REQUEST_DATA: ViewRequestData;
}

pub(crate) struct ViewRequestData {
    pub(crate) queries: Values,
    pub(crate) headers: Headers,
    pub(crate) cookies: Cookies,
    pub(crate) session: Session,
    pub(crate) parameters: Values,
}

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