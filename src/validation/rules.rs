use std::collections::HashMap;

use crate::{request::form::Form};

pub type Rule = dyn for<'a> Fn(&Form, String, Vec<String>) -> Option<String>;
pub type Rules = HashMap<String, Vec<Box<Rule>>>;

pub fn required(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.values.get(field.as_str()).is_none() && form.files.get(field.as_str()).is_none() {
        return Some(format!("The {} is required", field))
    }

    return None
}