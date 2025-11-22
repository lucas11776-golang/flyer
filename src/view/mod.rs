pub mod functions;
pub mod session_functions;

use std::io::Result;

use serde::{Serialize};
use tera::{Context, Tera};

use crate::{
    request::Request,
    response::Response,
    view::functions::Functions
};

pub(crate) struct View {
    pub(crate) render: Tera
}

pub struct ViewData {
    pub(crate) context: Context, 
}

impl ViewData {
    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.context.insert(key, val);
    }
}

pub fn view_data() -> ViewData {
    return ViewData{
        context: Context::new()
    };
}

impl View {
    pub fn new(path: &str) -> Self {
        return Self {
            render: Tera::new(&format!("{}/**/*", path.trim_end_matches("/"))).unwrap()
        }
    }

    pub fn render<'a>(&mut self, mut req: Request,  mut res: Response) -> Result<(Request, Response)> {
        if res.view.is_none() {
            return Ok((req, res));
        }

        let bag = res.view.as_mut().unwrap();

        if bag.data.is_none() {
            bag.data = Some(view_data());
        }

        res.body = Functions::new(&mut self.render, &mut req).render(bag).unwrap();

        return Ok((req, res));   
    }
}