use std::io::Result;

use serde::{Serialize};
use tera::{Context, Tera};

use crate::{request::Request, response::Response, view::functions::register};

pub(crate) mod functions;

pub(crate) struct ViewBag {
    pub(crate) view: String,
    pub(crate) data: Option<ViewData>,
}

pub struct ViewData {
    pub(crate) context: Context, 
}

impl ViewData {
    pub fn new() -> Self {
        return Self {
            context: Context::new()
        }
    }

    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.context.insert(key, val);
    }
}

#[deprecated]
pub fn view_data() -> ViewData {
    return ViewData{
        context: Context::new()
    };
}

pub(crate) struct View {
    pub(crate) engine: Tera
}

impl View {
    pub fn new(path: &str) -> Self {
        return Self {
            engine: Tera::new(&format!("{}/**/*", path.trim_end_matches("/"))).unwrap()
        }
    }

    pub fn render<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        return Ok(match res.view.as_mut() {
            Some(bag) => {
                if let None = bag.data {
                    bag.data = Some(ViewData::new());
                }

                register(&mut self.engine, req);

                res.body = self.engine.render(&bag.view, &bag.data.as_mut().unwrap().context).unwrap().into();
            },
            None => (),
        });
    }
}