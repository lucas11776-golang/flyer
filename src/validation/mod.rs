pub mod rules;

use std::{collections::HashMap, sync::LazyLock};

use async_std::task::block_on;

use crate::{
    request::{Request, form::{Files, Form}},
    response::Response,
    router::next::Next, utils::Values,
    validation::rules::{
        required,
        string,
        min,
        max,
        confirmed,
        file
    }
};

pub type Rule = dyn Fn(&Form, String, Vec<String>) -> Option<String> + 'static;

static mut RULES: LazyLock<HashMap<String, Box<Rule>>> = LazyLock::new(|| {
    let mut map: HashMap<String, Box<Rule>> = HashMap::new();

    map.insert(String::from("required"), Box::new(|form, field, args| block_on(required(form, field, args))));
    map.insert(String::from("string"), Box::new(|form, field, args| block_on(string(form, field, args))));
    map.insert(String::from("min"), Box::new(|form, field, args| block_on(min(form, field, args))));
    map.insert(String::from("max"), Box::new(|form, field, args| block_on(max(form, field, args))));
    map.insert(String::from("confirmed"), Box::new(|form, field, args| block_on(confirmed(form, field, args))));
    map.insert(String::from("file"), Box::new(|form, field, args| block_on(file(form, field, args))));

    return map;
});


pub struct Field {
    pub(crate) name: String,
    pub(crate) rules: Vec<(Box<Rule>, Vec<String>)>
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
        R: for<'a> AsyncFn(&'a Form, String, Vec<String>) -> Option<String> + Send + Sync + 'static
    {
        self.rules.push((
            Box::new(move |form, field, args| block_on(callback(form, field, args))),
            args.iter().map(|v| v.to_string()).collect()
        ));

        return self;
    }
}

pub struct Rules {
    pub(crate) fields: Vec<Field>
}



impl<'a> Rules {
    pub fn new() -> Rules {
        return Self {
            fields: Vec::new(),
        }
    }

    #[deprecated]
    pub fn field(&mut self, field: &str) -> &mut Field {
        let idx = self.fields.len();

        self.fields.push(Field::new(field));

        return &mut self.fields[idx];
    }

    pub fn rule(&mut self, field: &str, rules: Vec<&str>) -> &mut Self {
        return self;
    }

    #[allow(static_mut_refs)]
    pub fn add<R>(name: &str, callback: R)
    where
        R: for<'c> AsyncFn(&'c Form, String, Vec<String>) -> Option<String> + Send + Sync + 'static
    {
        unsafe {
            RULES.insert(String::from(name), Box::new(move |form, field, args| block_on(callback(form, field, args))));
        };
    }


    pub fn handle(&mut self, req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
        return next.handle(res);
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
        let mut validator = Self::new(&req.form, rules);

        if !validator.validate().await {
            return res.with_old(req.form.values.clone())
                .with_errors(validator.errors)
                .back();
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