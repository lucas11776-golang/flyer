use std::{fs::File, io::Read};

use anyhow::Result;
use bytes::Bytes;
use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    hooks::Hook,
    request::Request,
    response::Response,
    routing::next::Next,
    view::functions::register,
};

pub(crate) mod functions;

#[derive(Clone, Default)]
pub struct View {
    engine: Option<Tera>
}

impl Hook for View {
    #[allow(static_mut_refs)]
    async fn before(&self, req: Request, res: Response, next: Next) -> Response {
        return next.handle(req, res);
    }
    
    async fn after(&self, req: Request, mut res: Response, next: Next) -> Response {
        if let Some(engine) = &self.engine {
            if let Some(view) = res.view.as_mut() {
                let template_name = view
                    .view
                    .trim()
                    .trim_start_matches("/");

                let template = engine
                    .get_template(template_name)
                    .unwrap();

                let mut engine = Tera::default();

                engine.templates.insert(template_name.into(), template.clone());

                register(&mut engine, &req);

                res.content = self
                    .render_with_engine(&mut engine.clone(), view)
                    .unwrap()
                    .into();
            }
        }

        return next.handle(req, res);
    }
}

impl View {
    pub(crate) fn new(directory: Option<String>) -> Self {
        return match directory {
            Some(d) => Self {
                engine: Some(Tera::new(&format!("{}/**/*", d.trim_end_matches("/"))).unwrap()),
            },
            None => Self {
                engine: None
            },
        };
    }

    fn render_with_engine(&self, engine: &mut Tera, bag: &mut ViewBag) -> Result<String> {
        return engine
            .render(&bag.view, &bag.data.as_mut().unwrap_or(&mut ViewData::default()).context)
            .map_err(|err| anyhow::Error::from(err));
    }

    pub fn render(path: impl Into<String>, template: impl Into<String>, data: Option<ViewData>) -> Result<Bytes> {
        let filename = format!("{}/{}", path.into().trim_end_matches("/"), template.into().trim_start_matches(""));
        let mut template = String::new();

        File::open(filename)
            .unwrap()
            .read_to_string(&mut template)
            .unwrap();

        let context = data
            .map(|d| d.context)
            .unwrap_or(Context::new());

        return Tera::one_off(&template, &context, false)
            .map(|view| view.into())
            .map_err(|err| err.into());
    }
}

#[derive(Clone)]
pub(crate) struct ViewBag {
    pub(crate) view: String,
    pub(crate) data: Option<ViewData>,
}

impl ViewBag {
    pub fn new(view: impl Into<String>, data: Option<ViewData>) -> Self {
        return Self {
            view: view.into(),
            data: data,
        };
    }
}

#[derive(Clone, Default)]
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