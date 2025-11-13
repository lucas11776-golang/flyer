use std::{collections::HashMap, io::Result};

use tera::{Tera, Value, to_value};

use crate::{
    request::Request,
    response::ViewBag,
    utils::Values
};

pub(crate) struct Functions<'a> {
    render: &'a mut Tera,
    req: &'a mut Request,
}


// TODO: name according...
impl <'a>Functions<'a> {
    pub fn new(render: &'a mut Tera, req: &'a mut Request) -> Functions<'a> {
        return Self { 
            render: render,
            req: req,
        }
    }

    pub fn session(value: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            if args.get("name").is_none() {
                return Ok(to_value("")?);
            }

            let session = value.get(args.get("name").unwrap().as_str().unwrap());

            if session.is_none() {
                return Ok(to_value("")?);
            }

            return Ok(to_value(session.unwrap())?);
        };
    }

    pub fn session_has(value: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |args: &HashMap<String, Value>| -> tera::Result<tera::Value> {
            if args.get("name").is_none() {
                return Ok(to_value("")?);
            }

            return Ok(to_value(value.get(args.get("name").unwrap().as_str().unwrap()).is_some())?);
        };
    }

    pub fn error_has(_value: Values) -> impl Fn(&HashMap<String, Value>) -> tera::Result<tera::Value>  {
        return move |_args: &HashMap<String, Value>| -> tera::Result<tera::Value> {


            return Ok(to_value("Hello World")?);
        };
    }

    pub fn render(&mut self, bag: &'a mut ViewBag) -> Result<Vec<u8>> {
        if self.req.session.is_some() {
            self.render.register_function("session", Functions::session(self.req.session.as_mut().unwrap().values()));
            self.render.register_function("session_has", Functions::session_has(self.req.session.as_mut().unwrap().values()));
            self.render.register_function("error_has", Functions::error_has(self.req.session.as_mut().unwrap().errors()));
        }

        return Ok(
            self.render
                .render(&format!("{}", bag.view), &bag.data.as_mut().unwrap().context)
                .unwrap()
                .into()
        );
    }

}

