use std::collections::HashMap;

use crate::request::form::Form;

pub type Rule = dyn for<'a> Fn(&Form, String, Vec<String>) -> Option<String>;
pub type Rules = HashMap<String, Vec<Box<Rule>>>;

pub fn required(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_some() {
        return None
    }

    if form.values.get(field.as_str()).is_some() && form.values.get(field.as_str()).unwrap() != "" {
        return None
    }

    return Some(format!("The {} is required", field))
}

pub fn string(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.values.get(field.as_str()).is_none() {
        return Some(format!("The {} must be a string", field))
    }

    return None
}

pub fn min(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_some() {
        if form.files.get(field.as_str()).unwrap().content.len() < args[0].parse().unwrap() {
            return Some(format!("The {} must have minimum of {} kilobytes", field, args[0]));
        }

        return None
    }

    if form.values.get(field.as_str()).is_some() && form.values.get(field.as_str()).unwrap().len() >= args[0].parse().unwrap() {
        return None 
    }

    return Some(format!("The {} must have minimum of {} characters", field, args[0]));
}

pub fn file(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_none() {
        return Some(format!("The {} must be type of file", field))
    }
    
    return None
}