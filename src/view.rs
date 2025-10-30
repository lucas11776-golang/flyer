use std::io::Result;

use serde::Serialize;
use tera::{Context, Tera};

#[derive(Clone)]
pub struct View {
    pub(crate) render: Tera
}

#[derive(Clone)]
pub struct ViewData {
    pub(crate) context: Context, 
}

impl View {
    pub fn new(path: &str) -> Self {
        return Self {
            render: Tera::new(&format!("{}/**/*", path.trim_end_matches("/"))).unwrap()
        }
    }

    pub fn render(&mut self, view: &str, data: Option<ViewData>) -> String {
        match data {
            Some(data) => self.build(view, &data.context).unwrap(),
            None => self.build(view, &Context::new()).unwrap(),
        }
    }

    fn build(&mut self, view: &str, context: &Context) -> Result<String> {
        return Ok(self.render.render(&format!("{}", view), &context).unwrap())
    }
}

pub fn view_data() -> ViewData {
    return ViewData{
        context: Context::new()
    };
}

impl ViewData {
    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.context.insert(key, val);
    }
}
