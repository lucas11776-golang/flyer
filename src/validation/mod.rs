pub mod rules;

use std::collections::HashMap;

use crate::{
    request::{Request, form::{Files, Form}},
    response::Response,
    router::next::Next, utils::Values
};

pub type Rule = fn (&Form, String, Vec<String>) -> Option<String>;
pub type Rules = HashMap<String, Vec<(Rule, Vec<String>)>>;

#[allow(dead_code)]
pub struct Validator<'a> {
    pub(crate) form: &'a Form,
    pub(crate) rules: Rules,
    pub(crate) errors: Values,
    pub(crate) validated: Form,
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
        for (field, validators) in  &mut self.rules {
            if let Some(error) = Self::validate_field(self.form, field.to_string(), validators).await {
                self.errors.insert(field.to_string(), error);
            }
        }

        return self.errors.len() == 0;
    }

    pub async fn handle(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next, rules: Rules) -> &'a mut Response {
        let mut validator = Self::new(&req.form, rules);
        println!("FORM: {:?}", req.form.values);

        if !validator.validate().await {
            return res.with_errors(validator.errors).back();
        }

        return next.handle(res);
    }

    pub fn errors(&mut self) -> Values {
        return Values::new();
    }

    async fn validate_field(form: &Form, field: String, validators: &mut Vec<(Rule, Vec<String>)>) -> Option<String> {
        for (rule, args) in validators {
            if let Some(error) = rule(form, field.to_string(), args.to_vec()) {
                return Some(error)
            }
        }

        return None
    }
}