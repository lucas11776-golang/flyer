use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    hooks::Hook,
    request::Request,
    response::Response,
    routing::next::Next,
    view::functions::{
        HelperFunctions,
        VIEW_REQUEST_DATA,
        ViewRequestData,
        request::RequestHelperFunctions,
        utils::UtilsHelperFunctions
    }
};

pub(crate) mod functions;

#[derive(Clone, Default)]
pub struct View {
    engine: Option<Arc<Tera>>,
}

impl Hook for View {
    async fn before(&self, req: Request, res: Response, next: Next) -> Response {
        next.handle(req, res)
    }

    async fn after(&self, req: Request, mut res: Response, next: Next) -> Response {
        if self.engine.is_none() && res.view.is_none() {
            return next.handle(req, res);
        }

        let engine = self
            .engine
            .as_ref()
            .unwrap();

        let mut view = res
            .view
            .take()
            .unwrap();

        let view_request_data = ViewRequestData {
            queries: req.queries.clone(),
            headers: req.headers.clone(),
            cookies: req.cookies.clone(),
            session: req.session.clone(),
            parameters: req.parameters.clone(),
        };

        res.content = VIEW_REQUEST_DATA
            .scope(view_request_data, async { 
                self.render_with_engine(engine, &mut view)
            })
            .await
            .unwrap()
            .into();

        next.handle(req, res)
    }
}

impl View {
    pub(crate) fn new(directory: Option<impl Into<String>>) -> Self {
        let engine = directory.map(|dir| {
            let view_path = format!("{}/**/*", dir.into().trim_end_matches('/'));
            let mut engine = Tera::new(&view_path)
                .unwrap();

            RequestHelperFunctions::register(&mut engine);
            UtilsHelperFunctions::register(&mut engine);

            Arc::new(engine)
        });

        Self { engine }
    }

    fn render_with_engine(&self, engine: &Tera, bag: &mut ViewBag) -> Result<String> {
        let default_data = &mut ViewData::default();
        let context = &bag.data.as_mut().unwrap_or(default_data).context;

        engine
            .render(&bag.view, context)
            .map_err(|err| err.into())
    }

    // TODO: need to be path only.
    pub fn render(path: impl Into<String>, template: impl Into<String>, data: Option<ViewData>) -> Result<Bytes> {
        let filename = format!(
            "{}/{}",
            path.into().trim_end_matches('/'),
            template.into().trim_start_matches('/')
        );

        let template = std::fs::read_to_string(filename)?;

        let context = data
            .map(|d| d.context)
            .unwrap_or_default();

        let mut engine = Tera::default();

        UtilsHelperFunctions::register(&mut engine);

        engine
            .render_str(&template, &context)
            .map(|v| v.into())
            .map_err(|err| err.into())
    }
}

#[derive(Clone)]
pub(crate) struct ViewBag {
    pub(crate) view: String,
    pub(crate) data: Option<ViewData>,
}

impl Serialize for ViewBag {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut map: HashMap<String, serde_json::Value> = Default::default();

        map.insert("view".into(), self.view.clone().into());

        serializer.collect_map(map)
    }
}

impl ViewBag {
    pub fn new(view: impl Into<String>, data: Option<ViewData>) -> Self {
        Self {
            view: view.into(),
            data: data,
        }
    }
}

#[derive(Clone, Default)]
pub struct ViewData {
    pub(crate) context: Context,
}

impl ViewData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<T: Serialize + ?Sized, S: Into<String>>(key: S, val: &T) -> Self {
        let mut data = Self::new();
        data.insert(key, val);
        data
    }

    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) -> &mut Self {
        self.context.insert(key, val);
        self
    }
}