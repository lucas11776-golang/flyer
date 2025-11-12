use std::io::Result;

use serde::Serialize;
use tera::{Context, Tera};

use crate::response::Response;

pub(crate) struct View {
    pub(crate) render: Tera
}

#[derive(Clone)]
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

    pub fn render<'a>(&mut self, mut res: Response) -> Result<Response> {
        if res.view.is_none() {
            return Ok(res);
        }

        let bag = res.view.as_mut().unwrap();

        if bag.data.is_none() {
            bag.data = Some(view_data());
        }

        res.body = self.render
            .render(&format!("{}", bag.view), &bag.data.as_mut().unwrap().context)
            .unwrap()
            .into();

        return Ok(res);   
    }
}