use crate::request::form::Form;
use regex::Regex;

fn pretty(value: String) -> String {
    let temp: Vec<&str> = value.split('_').collect();
    temp.join(" ")
}

pub async fn required(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_some() || (form.values.get(field.as_str()).is_some() && !form.values.get(field.as_str()).unwrap().is_empty()) {
        return None;
    }
    Some(format!("The {} is required", pretty(field)))
}

pub async fn string(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.values.get(field.as_str()).is_none() {
        return Some(format!("The {} must be a string", pretty(field)));
    }
    None
}

pub async fn alpha(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    if val.chars().all(|c| c.is_alphabetic()) { return None; }
    Some("Only alphabetic characters are allowed".to_string())
}

pub async fn alpha_numeric(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    if val.chars().all(|c| c.is_alphanumeric()) { return None; }
    Some("Only alphanumeric characters are allowed".to_string())
}

pub async fn email(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if email_regex.is_match(val) { return None; }
    Some("Invalid email address".to_string())
}

pub async fn min_length(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    let min: usize = args[0].parse().unwrap_or(0);
    if val.len() >= min { return None; }
    Some(format!("Minimum length is {} characters", min))
}

pub async fn max_length(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    let max: usize = args[0].parse().unwrap_or(0);
    if val.len() <= max { return None; }
    Some(format!("Maximum length is {} characters", max))
}

pub async fn numeric(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    if val.chars().all(|c| c.is_numeric()) { return None; }
    Some("Only numeric values are allowed".to_string())
}

use reqwest::Url;

pub async fn url(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let val = form.values.get(field.as_str())?;
    if Url::parse(val).is_ok() { return None; }
    Some("Invalid URL format".to_string())
}

pub async fn min(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(file) = form.files.get(field.as_str()) {
        if file.content.len() < args[0].parse().unwrap_or(0) {
            return Some(format!("The {} must have minimum of {} kilobytes", pretty(field), args[0]));
        }
        return None;
    }
    if let Some(val) = form.values.get(field.as_str()) {
        if val.len() >= args[0].parse().unwrap_or(0) { return None; }
    }
    Some(format!("The {} must have minimum of {} characters", pretty(field), args[0]))
}

pub async fn max(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(file) = form.files.get(field.as_str()) {
        if file.content.len() > args[0].parse().unwrap_or(0) {
            return Some(format!("The {} must have minimum of {} kilobytes", pretty(field), args[0]));
        }
        return None;
    }
    if let Some(val) = form.values.get(field.as_str()) {
        if val.len() <= args[0].parse().unwrap_or(0) { return None; }
    }
    Some(format!("The {} must have minimum of {} characters", pretty(field), args[0]))
}

pub async fn confirmed(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = form.values.get(&field) {
        if let Some(confirm) = form.values.get(&format!("{}_confirmed", field)) {
            if confirm == val { return None; }
        }
    }
    Some(format!("The {} does not match {} confirmation", pretty(field.clone()), pretty(field)))
}

pub async fn file(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_none() {
        return Some(format!("The {} must be type of file", pretty(field)));
    }
    None
}
