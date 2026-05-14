use anyhow::Result;
use serde::{Serialize};
use tera::{Context, Tera};

use crate::{GLOBAL_SERVER, request::Request, response::Response, view::functions::{register, register_utils_functions}};

pub(crate) mod functions;

pub(crate) struct ViewBag {
    pub(crate) view: String,
    pub(crate) data: Option<ViewData>,
}

#[derive(Default)]
pub struct ViewData {
    pub(crate) context: Context, 
}

impl ViewData {
    pub fn new() -> Self {
        return Self {
            context: Context::new()
        }
    }

    pub fn with<T: Serialize + ?Sized, S: Into<String>>(key: S, val: &T) -> ViewData {
        let mut data = Self::new();
        data.insert(key, val);
        return data;
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
        if let Some(bag) = res.view.as_mut() {
            register(&mut self.engine, req);
            
            res.body = self.render_view_bag(bag).unwrap().into();
        }
        return Ok(());
    }

    fn render_view_bag(&mut self, bag: &mut ViewBag) -> Result<String> {
        return self.engine
            .render(&bag.view, &bag.data.as_mut().unwrap_or(&mut ViewData::default()).context)
            .map_err(|err| anyhow::Error::from(err));
    }

    fn render_view(&mut self, template: &str, data: Option<ViewData>) -> Result<String> {
        register_utils_functions(&mut self.engine);

        return self.engine
            .render(template, &data.unwrap_or(ViewData::default()).context)
            .map_err(|err| anyhow::Error::from(err));
    }
}

#[allow(static_mut_refs)]
pub fn render_view(path: &str, data: Option<ViewData>) -> Result<String> {
    unsafe {
        return GLOBAL_SERVER.get_mut()
            .unwrap()
            .view
            .as_mut()
            .unwrap()
            .render_view(path, data)
    };
}