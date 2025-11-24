pub mod rules;

use async_std::task::block_on;

use crate::{
    request::{Request, form::{Files, Form}},
    response::Response,
    router::next::Next, utils::Values
};

pub type Rule = dyn Fn(&Form, String, Vec<String>) -> Option<String> + 'static;

pub struct Field {
    pub(crate) name: String,
    pub(crate) rules: Vec<(Box<Rule>, Vec<String>)>
}

pub struct Rules {
    pub(crate) fields: Vec<Field>
}

#[allow(dead_code)]
pub struct Validator<'a> {
    pub(crate) form: &'a Form,
    pub(crate) rules: Rules,
    pub(crate) errors: Values,
    pub(crate) validated: Form,
}

impl Field {
    pub(crate) fn new(field: &str) -> Field {
        return Self {
            name: field.to_string(),
            rules: Vec::new()
        }
    }

    pub fn add<R>(&mut self, callback: R, args: Vec<&str>) -> &mut Field
    where
        R: for<'a> AsyncFn<(&'a Form, String, Vec<String>), Output = Option<String>> + Send + Sync + 'static
    {
        self.rules.push((
            Box::new(move |form, field, args| block_on(callback(form, field, args))),
            args.iter().map(|v| v.to_string()).collect()
        ));

        return self;
    }
}

impl<'a> Rules {
    pub fn new() -> Rules {
        return Self {
            fields: Vec::new(),
        }
    }

    pub fn field(&mut self, field: &str) -> &mut Field {
        let idx = self.fields.len();

        self.fields.push(Field::new(field));

        return &mut self.fields[idx];
    }
}

impl <'a>Validator<'a> {
    pub fn new(form: &'a Form, rules: Rules) -> Self {
        return Self {
            form: form,
            rules: rules,
            errors: Values::new(),
            validated: Form::new(Values::new(), Files::new())
        }
    }

    pub async fn validate(&mut self) -> bool {
        for field in &mut self.rules.fields {
            if let Some(error) = Self::validate_field(self.form, field) {
                self.errors.insert(field.name.to_string(), error);
            }
        }

        return self.errors.len() == 0;
    }

    pub async fn handle(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next, rules: Rules) -> &'a mut Response {
        let session = req.session();
        let mut validator = Self::new(&req.form, rules);

        if !validator.validate().await {

            // session.set_old(req.form.values.clone());

            return res.with_errors(validator.errors).back();
        }

        return next.handle(res);
    }

    pub fn errors(&mut self) -> Values {
        return Values::new();
    }


    fn validate_field(form: &Form, field: &mut Field) -> Option<String> {
        for (rule, args) in &mut field.rules {
            if let Some(error) = rule(form, field.name.clone(), args.to_vec()) {
                return Some(error)
            }
        }

        return None
    }
}