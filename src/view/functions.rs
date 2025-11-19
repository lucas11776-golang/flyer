use std::io::Result;

use tera::Tera;

use crate::{
    request::Request,
    response::ViewBag,
    view::session_functions::SessionFunctions
};

pub(crate) struct Functions<'a> {
    render: &'a mut Tera,
    req: &'a mut Request,
}

impl <'a>Functions<'a> {
    pub fn new(render: &'a mut Tera, req: &'a mut Request) -> Functions<'a> {
        return Self { 
            render: render,
            req: req,
        }
    }

    fn register_session_functions(&mut self) {
        self.render.register_function("session", SessionFunctions::session(self.req.session.as_mut().unwrap().values()));
        self.render.register_function("session_has", SessionFunctions::session_has(self.req.session.as_mut().unwrap().values()));
        self.render.register_function("error_has", SessionFunctions::error_has(self.req.session.as_mut().unwrap().errors()));
        self.render.register_function("error", SessionFunctions::error(self.req.session.as_mut().unwrap().errors()));
    }

    pub fn render(&mut self, bag: &'a mut ViewBag) -> Result<Vec<u8>> {
        self.register_session_functions();

        let rendered = self.render
            .render(&format!("{}", bag.view), &bag.data.as_mut().unwrap().context)
            .unwrap()
            .into();

        return Ok(rendered);
    }

}