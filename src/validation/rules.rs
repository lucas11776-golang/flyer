use crate::request::form::Form;
use regex::Regex;
use chrono::{NaiveDate, NaiveDateTime};
use std::net::IpAddr;
use std::str::FromStr;
use reqwest::Url;
use uuid::Uuid;
use ulid::Ulid;
use serde_json::Value as JsonValue;

fn pretty(value: String) -> String {
    let temp: Vec<&str> = value.split('_').collect();
    temp.join(" ")
}

fn get_value(form: &Form, field: &str) -> Option<String> {
    form.values.get(field).cloned()
}

fn is_empty(form: &Form, field: &str) -> bool {
    if let Some(val) = form.values.get(field) {
        return val.is_empty();
    }
    if form.files.get(field).is_some() {
        return false;
    }
    true
}

fn is_present(form: &Form, field: &str) -> bool {
    form.values.contains_key(field) || form.files.contains_key(field)
}

// Booleans
pub async fn accepted(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        let accepted_vals = vec!["yes", "on", "1", "true"];
        if accepted_vals.contains(&val.to_lowercase().as_str()) || val == "1" {
            return None;
        }
    }
    Some(format!("The {} must be accepted", pretty(field)))
}

pub async fn accepted_if(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];

    if let Some(val) = get_value(form, another_field) {
        if val == *expected_val {
            return accepted(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn boolean(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        let bool_vals = vec!["true", "false", "1", "0", "on", "off", "yes", "no"];
        if bool_vals.contains(&val.to_lowercase().as_str()) {
            return None;
        }
    }
    Some(format!("The {} must be a boolean", pretty(field)))
}

pub async fn declined(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        let declined_vals = vec!["no", "off", "0", "false"];
        if declined_vals.contains(&val.to_lowercase().as_str()) {
            return None;
        }
    }
    Some(format!("The {} must be declined", pretty(field)))
}

pub async fn declined_if(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];

    if let Some(val) = get_value(form, another_field) {
        if val == *expected_val {
            return declined(form, field, Vec::new()).await;
        }
    }
    None
}

// Strings
pub async fn active_url(_form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(_form, &field) {
        if let Ok(url) = Url::parse(&val) {
            if let Some(host) = url.host_str() {
                if tokio::net::lookup_host(format!("{}:80", host)).await.is_ok() {
                    return None;
                }
            }
        }
    }
    Some(format!("The {} is not a valid active URL", pretty(field)))
}

pub async fn alpha(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if !val.is_empty() && val.chars().all(|c| c.is_alphabetic()) {
            return None;
        }
    }
    Some(format!("The {} must only contain alphabetic characters", pretty(field)))
}

pub async fn alpha_dash(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if !val.is_empty() && val.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return None;
        }
    }
    Some(format!("The {} must only contain letters, numbers, dashes and underscores", pretty(field)))
}

pub async fn alpha_numeric(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if !val.is_empty() && val.chars().all(|c| c.is_alphanumeric()) {
            return None;
        }
    }
    Some(format!("The {} must only contain alphanumeric characters", pretty(field)))
}

pub async fn ascii(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if val.is_ascii() {
            return None;
        }
    }
    Some(format!("The {} must only contain ASCII characters", pretty(field)))
}

pub async fn confirmed(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    let confirmation_field = format!("{}_confirmation", field);
    if let Some(val) = get_value(form, &field) {
        if let Some(conf) = get_value(form, &confirmation_field) {
            if val == conf {
                return None;
            }
        }
    }
    Some(format!("The {} confirmation does not match", pretty(field)))
}

pub async fn different(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if let Some(val) = get_value(form, &field) {
        if let Some(another_val) = get_value(form, another_field) {
            if val != another_val {
                return None;
            }
        } else {
            return None; 
        }
    }
    Some(format!("The {} must be different from {}", pretty(field), pretty(another_field.clone())))
}

pub async fn doesnt_start_with(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        for arg in &args {
            if val.starts_with(arg) {
                return Some(format!("The {} must not start with {}", pretty(field), arg));
            }
        }
    }
    None
}

pub async fn doesnt_end_with(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        for arg in &args {
            if val.ends_with(arg) {
                return Some(format!("The {} must not end with {}", pretty(field), arg));
            }
        }
    }
    None
}

pub async fn email(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if email_regex.is_match(&val) { return None; }
    }
    Some(format!("The {} must be a valid email address", pretty(field)))
}

pub async fn ends_with(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        for arg in &args {
            if val.ends_with(arg) {
                return None;
            }
        }
    }
    Some(format!("The {} must end with one of: {}", pretty(field), args.join(", ")))
}

pub async fn hex_color(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        let hex_regex = Regex::new(r"^#?([a-fA-F0-9]{3}|[a-fA-F0-9]{6})$").unwrap();
        if hex_regex.is_match(&val) { return None; }
    }
    Some(format!("The {} must be a valid hexadecimal color", pretty(field)))
}

pub async fn in_rule(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if args.contains(&val) {
            return None;
        }
    }
    Some(format!("The selected {} is invalid", pretty(field)))
}

pub async fn ip(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if IpAddr::from_str(&val).is_ok() { return None; }
    }
    Some(format!("The {} must be a valid IP address", pretty(field)))
}

pub async fn ipv4(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if let Ok(IpAddr::V4(_)) = IpAddr::from_str(&val) { return None; }
    }
    Some(format!("The {} must be a valid IPv4 address", pretty(field)))
}

pub async fn ipv6(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if let Ok(IpAddr::V6(_)) = IpAddr::from_str(&val) { return None; }
    }
    Some(format!("The {} must be a valid IPv6 address", pretty(field)))
}

pub async fn json(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if serde_json::from_str::<JsonValue>(&val).is_ok() { return None; }
    }
    Some(format!("The {} must be a valid JSON string", pretty(field)))
}

pub async fn lowercase(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if val == val.to_lowercase() { return None; }
    }
    Some(format!("The {} must be lowercase", pretty(field)))
}

pub async fn mac_address(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        let mac_regex = Regex::new(r"^([0-9a-fA-F]{2}[:-]){5}([0-9a-fA-F]{2})$").unwrap();
        if mac_regex.is_match(&val) { return None; }
    }
    Some(format!("The {} must be a valid MAC address", pretty(field)))
}

pub async fn not_in(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if !args.contains(&val) {
            return None;
        }
    }
    Some(format!("The selected {} is invalid", pretty(field)))
}

pub async fn regex(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    if let Some(val) = get_value(form, &field) {
        if let Ok(re) = Regex::new(&args[0]) {
            if re.is_match(&val) { return None; }
        }
    }
    Some(format!("The {} format is invalid", pretty(field)))
}

pub async fn not_regex(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    if let Some(val) = get_value(form, &field) {
        if let Ok(re) = Regex::new(&args[0]) {
            if !re.is_match(&val) { return None; }
        }
    }
    Some(format!("The {} format is invalid", pretty(field)))
}

pub async fn same(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if let Some(val) = get_value(form, &field) {
        if let Some(another_val) = get_value(form, another_field) {
            if val == another_val { return None; }
        }
    }
    Some(format!("The {} and {} must match", pretty(field), pretty(another_field.clone())))
}

pub async fn starts_with(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        for arg in &args {
            if val.starts_with(arg) {
                return None;
            }
        }
    }
    Some(format!("The {} must start with one of: {}", pretty(field), args.join(", ")))
}

pub async fn uppercase(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if val == val.to_uppercase() { return None; }
    }
    Some(format!("The {} must be uppercase", pretty(field)))
}

pub async fn url(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if Url::parse(&val).is_ok() { return None; }
    }
    Some(format!("The {} must be a valid URL", pretty(field)))
}

pub async fn uuid(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if Uuid::parse_str(&val).is_ok() { return None; }
    }
    Some(format!("The {} must be a valid UUID", pretty(field)))
}

pub async fn ulid(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if Ulid::from_str(&val).is_ok() { return None; }
    }
    Some(format!("The {} must be a valid ULID", pretty(field)))
}

// Numbers
pub async fn between(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let min: f64 = args[0].parse().unwrap_or(0.0);
    let max: f64 = args[1].parse().unwrap_or(0.0);

    if let Some(val) = get_value(form, &field) {
        if let Ok(num) = val.parse::<f64>() {
            if num >= min && num <= max { return None; }
            return Some(format!("The {} must be between {} and {}", pretty(field), min, max));
        }
        if val.len() >= min as usize && val.len() <= max as usize { return None; }
        return Some(format!("The {} must be between {} and {} characters", pretty(field), min, max));
    }
    if let Some(file) = form.files.get(&field) {
        let size_kb = file.content.len() / 1024;
        if size_kb >= min as usize && size_kb <= max as usize { return None; }
        return Some(format!("The {} must be between {} and {} kilobytes", pretty(field), min, max));
    }
    None
}

pub async fn decimal(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if let Ok(_) = val.parse::<f64>() {
            if val.contains('.') {
                let parts: Vec<&str> = val.split('.').collect();
                if parts.len() == 2 {
                    let decimal_places = parts[1].len();
                    if args.len() >= 1 {
                        let min: usize = args[0].parse().unwrap_or(0);
                        if args.len() >= 2 {
                             let max: usize = args[1].parse().unwrap_or(min);
                             if decimal_places >= min && decimal_places <= max { return None; }
                             return Some(format!("The {} must have between {} and {} decimal places", pretty(field), min, max));
                        }
                        if decimal_places == min { return None; }
                        return Some(format!("The {} must have {} decimal places", pretty(field), min));
                    }
                    return None;
                }
            }
        }
    }
    Some(format!("The {} must be a decimal", pretty(field)))
}

pub async fn digits(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let len: usize = args[0].parse().unwrap_or(0);
    if let Some(val) = get_value(form, &field) {
        if val.chars().all(|c| c.is_numeric()) && val.len() == len {
            return None;
        }
    }
    Some(format!("The {} must be {} digits", pretty(field), len))
}

pub async fn digits_between(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let min: usize = args[0].parse().unwrap_or(0);
    let max: usize = args[1].parse().unwrap_or(0);
    if let Some(val) = get_value(form, &field) {
        if val.chars().all(|c| c.is_numeric()) && val.len() >= min && val.len() <= max {
            return None;
        }
    }
    Some(format!("The {} must be between {} and {} digits", pretty(field), min, max))
}

pub async fn gt(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if let Some(val) = get_value(form, &field) {
        if let Ok(v1) = val.parse::<f64>() {
            if let Some(another_val) = get_value(form, another_field) {
                if let Ok(v2) = another_val.parse::<f64>() {
                    if v1 > v2 { return None; }
                }
            } else if let Ok(v2) = another_field.parse::<f64>() {
                 if v1 > v2 { return None; }
            }
        }
    }
    Some(format!("The {} must be greater than {}", pretty(field), pretty(another_field.clone())))
}

pub async fn gte(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if let Some(val) = get_value(form, &field) {
        if let Ok(v1) = val.parse::<f64>() {
            if let Some(another_val) = get_value(form, another_field) {
                if let Ok(v2) = another_val.parse::<f64>() {
                    if v1 >= v2 { return None; }
                }
            } else if let Ok(v2) = another_field.parse::<f64>() {
                 if v1 >= v2 { return None; }
            }
        }
    }
    Some(format!("The {} must be greater than or equal {}", pretty(field), pretty(another_field.clone())))
}

pub async fn integer(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if val.parse::<i128>().is_ok() { return None; }
    }
    Some(format!("The {} must be an integer", pretty(field)))
}

pub async fn lt(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if let Some(val) = get_value(form, &field) {
        if let Ok(v1) = val.parse::<f64>() {
            if let Some(another_val) = get_value(form, another_field) {
                if let Ok(v2) = another_val.parse::<f64>() {
                    if v1 < v2 { return None; }
                }
            } else if let Ok(v2) = another_field.parse::<f64>() {
                 if v1 < v2 { return None; }
            }
        }
    }
    Some(format!("The {} must be less than {}", pretty(field), pretty(another_field.clone())))
}

pub async fn lte(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if let Some(val) = get_value(form, &field) {
        if let Ok(v1) = val.parse::<f64>() {
            if let Some(another_val) = get_value(form, another_field) {
                if let Ok(v2) = another_val.parse::<f64>() {
                    if v1 <= v2 { return None; }
                }
            } else if let Ok(v2) = another_field.parse::<f64>() {
                 if v1 <= v2 { return None; }
            }
        }
    }
    Some(format!("The {} must be less than or equal {}", pretty(field), pretty(another_field.clone())))
}

pub async fn max(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let max_val: f64 = args[0].parse().unwrap_or(0.0);

    if let Some(val) = get_value(form, &field) {
        if let Ok(num) = val.parse::<f64>() {
            if num <= max_val { return None; }
            return Some(format!("The {} must not be greater than {}", pretty(field), max_val));
        }
        if val.len() <= max_val as usize { return None; }
        return Some(format!("The {} must not be greater than {} characters", pretty(field), max_val));
    }
    if let Some(file) = form.files.get(&field) {
        let size_kb = file.content.len() / 1024;
        if size_kb <= max_val as usize { return None; }
        return Some(format!("The {} must not be greater than {} kilobytes", pretty(field), max_val));
    }
    None
}

pub async fn min(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let min_val: f64 = args[0].parse().unwrap_or(0.0);

    if let Some(val) = get_value(form, &field) {
        if let Ok(num) = val.parse::<f64>() {
            if num >= min_val { return None; }
            return Some(format!("The {} must be at least {}", pretty(field), min_val));
        }
        if val.len() >= min_val as usize { return None; }
        return Some(format!("The {} must be at least {} characters", pretty(field), min_val));
    }
    if let Some(file) = form.files.get(&field) {
        let size_kb = file.content.len() / 1024;
        if size_kb >= min_val as usize { return None; }
        return Some(format!("The {} must be at least {} kilobytes", pretty(field), min_val));
    }
    None
}

pub async fn max_digits(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let max: usize = args[0].parse().unwrap_or(0);
    if let Some(val) = get_value(form, &field) {
        if val.chars().all(|c| c.is_numeric()) && val.len() <= max {
            return None;
        }
    }
    Some(format!("The {} must not have more than {} digits", pretty(field), max))
}

pub async fn min_digits(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let min: usize = args[0].parse().unwrap_or(0);
    if let Some(val) = get_value(form, &field) {
        if val.chars().all(|c| c.is_numeric()) && val.len() >= min {
            return None;
        }
    }
    Some(format!("The {} must have at least {} digits", pretty(field), min))
}

pub async fn multiple_of(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let factor: f64 = args[0].parse().unwrap_or(1.0);
    if let Some(val) = get_value(form, &field) {
        if let Ok(num) = val.parse::<f64>() {
            if num % factor == 0.0 { return None; }
        }
    }
    Some(format!("The {} must be a multiple of {}", pretty(field), factor))
}

pub async fn numeric(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if val.parse::<f64>().is_ok() { return None; }
    }
    Some(format!("The {} must be a number", pretty(field)))
}

// Dates
fn parse_date(date_str: &str) -> Option<NaiveDateTime> {
    let formats = vec![
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
        "%d-%m-%Y",
        "%m/%d/%Y",
    ];
    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, format) {
            return Some(dt);
        }
        if let Ok(d) = NaiveDate::parse_from_str(date_str, format) {
            return Some(d.and_hms_opt(0, 0, 0).unwrap());
        }
    }
    None
}

pub async fn after(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let target_date_str = if let Some(val) = get_value(form, &args[0]) { val } else { args[0].clone() };
    if let Some(val) = get_value(form, &field) {
        if let (Some(d1), Some(d2)) = (parse_date(&val), parse_date(&target_date_str)) {
            if d1 > d2 { return None; }
        }
    }
    Some(format!("The {} must be a date after {}", pretty(field), target_date_str))
}

pub async fn after_or_equal(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let target_date_str = if let Some(val) = get_value(form, &args[0]) { val } else { args[0].clone() };
    if let Some(val) = get_value(form, &field) {
        if let (Some(d1), Some(d2)) = (parse_date(&val), parse_date(&target_date_str)) {
            if d1 >= d2 { return None; }
        }
    }
    Some(format!("The {} must be a date after or equal to {}", pretty(field), target_date_str))
}

pub async fn before(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let target_date_str = if let Some(val) = get_value(form, &args[0]) { val } else { args[0].clone() };
    if let Some(val) = get_value(form, &field) {
        if let (Some(d1), Some(d2)) = (parse_date(&val), parse_date(&target_date_str)) {
            if d1 < d2 { return None; }
        }
    }
    Some(format!("The {} must be a date before {}", pretty(field), target_date_str))
}

pub async fn before_or_equal(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let target_date_str = if let Some(val) = get_value(form, &args[0]) { val } else { args[0].clone() };
    if let Some(val) = get_value(form, &field) {
        if let (Some(d1), Some(d2)) = (parse_date(&val), parse_date(&target_date_str)) {
            if d1 <= d2 { return None; }
        }
    }
    Some(format!("The {} must be a date before or equal to {}", pretty(field), target_date_str))
}

pub async fn date(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(val) = get_value(form, &field) {
        if parse_date(&val).is_some() { return None; }
    }
    Some(format!("The {} is not a valid date", pretty(field)))
}

pub async fn date_equals(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let target_date_str = if let Some(val) = get_value(form, &args[0]) { val } else { args[0].clone() };
    if let Some(val) = get_value(form, &field) {
        if let (Some(d1), Some(d2)) = (parse_date(&val), parse_date(&target_date_str)) {
            if d1 == d2 { return None; }
        }
    }
    Some(format!("The {} must be a date equal to {}", pretty(field), target_date_str))
}

pub async fn date_format(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let format = &args[0];
    if let Some(val) = get_value(form, &field) {
        if NaiveDateTime::parse_from_str(&val, format).is_ok() || NaiveDate::parse_from_str(&val, format).is_ok() {
            return None;
        }
    }
    Some(format!("The {} does not match the format {}", pretty(field), format))
}

// Files
pub async fn file(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(&field).is_some() { return None; }
    Some(format!("The {} must be a file", pretty(field)))
}

pub async fn image(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(file) = form.files.get(&field) {
        let image_mimes = vec!["image/jpeg", "image/png", "image/gif", "image/bmp", "image/svg+xml", "image/webp"];
        if image_mimes.contains(&file.mime.as_str()) {
            return None;
        }
    }
    Some(format!("The {} must be an image", pretty(field)))
}

pub async fn mimetypes(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(file) = form.files.get(&field) {
        if args.contains(&file.mime) {
            return None;
        }
    }
    Some(format!("The {} must be a file of type: {}", pretty(field), args.join(", ")))
}

pub async fn mimes(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if let Some(file) = form.files.get(&field) {
        let parts: Vec<&str> = file.name.split('.').collect();
        if let Some(ext) = parts.last() {
            if args.contains(&ext.to_string()) {
                return None;
            }
        }
    }
    Some(format!("The {} must be a file of type: {}", pretty(field), args.join(", ")))
}

pub async fn extensions(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    mimes(form, field, args).await
}

// Utilities
pub async fn required(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if !is_empty(form, &field) { return None; }
    Some(format!("The {} field is required", pretty(field)))
}

pub async fn required_if(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];

    if let Some(val) = get_value(form, another_field) {
        if val == *expected_val {
            return required(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn required_if_accepted(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let another_field = &args[0];
    if accepted(form, another_field.clone(), Vec::new()).await.is_none() {
        return required(form, field, Vec::new()).await;
    }
    None
}

pub async fn required_unless(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];

    if let Some(val) = get_value(form, another_field) {
        if val != *expected_val {
            return required(form, field, Vec::new()).await;
        }
    } else {
        return required(form, field, Vec::new()).await;
    }
    None
}

pub async fn required_with(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    for arg in &args {
        if !is_empty(form, arg) {
            return required(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn required_without(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    for arg in &args {
        if is_empty(form, arg) {
            return required(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn required_with_all(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    for arg in &args {
        if is_empty(form, arg) {
            return None;
        }
    }
    required(form, field, Vec::new()).await
}

pub async fn required_without_all(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    for arg in &args {
        if !is_empty(form, arg) {
            return None;
        }
    }
    required(form, field, Vec::new()).await
}

pub async fn prohibited(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if is_empty(form, &field) { return None; }
    Some(format!("The {} field is prohibited", pretty(field)))
}

pub async fn prohibited_if(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];

    if let Some(val) = get_value(form, another_field) {
        if val == *expected_val {
            return prohibited(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn prohibited_unless(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];

    if let Some(val) = get_value(form, another_field) {
        if val != *expected_val {
            return prohibited(form, field, Vec::new()).await;
        }
    } else {
        return prohibited(form, field, Vec::new()).await;
    }
    None
}

pub async fn prohibited_with(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    for arg in &args {
        if !is_empty(form, arg) {
            return prohibited(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn prohibited_with_all(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    for arg in &args {
        if is_empty(form, arg) {
            return None;
        }
    }
    prohibited(form, field, Vec::new()).await
}

pub async fn filled(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if is_present(form, &field) && is_empty(form, &field) {
        return Some(format!("The {} field must have a value", pretty(field)));
    }
    None
}

pub async fn missing(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if is_present(form, &field) {
        return Some(format!("The {} field must be missing", pretty(field)));
    }
    None
}

pub async fn missing_if(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];
    if let Some(val) = get_value(form, another_field) {
        if val == *expected_val {
            return missing(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn missing_unless(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];
    if let Some(val) = get_value(form, another_field) {
        if val != *expected_val {
            return missing(form, field, Vec::new()).await;
        }
    } else {
        return missing(form, field, Vec::new()).await;
    }
    None
}

pub async fn present(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if !is_present(form, &field) {
        return Some(format!("The {} field must be present", pretty(field)));
    }
    None
}

pub async fn present_if(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];
    if let Some(val) = get_value(form, another_field) {
        if val == *expected_val {
            return present(form, field, Vec::new()).await;
        }
    }
    None
}

pub async fn present_unless(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.len() < 2 { return None; }
    let another_field = &args[0];
    let expected_val = &args[1];
    if let Some(val) = get_value(form, another_field) {
        if val != *expected_val {
            return present(form, field, Vec::new()).await;
        }
    } else {
        return present(form, field, Vec::new()).await;
    }
    None
}

pub async fn string(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if get_value(form, &field).is_some() { return None; }
    Some(format!("The {} must be a string", pretty(field)))
}

pub async fn size(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if args.is_empty() { return None; }
    let size: usize = args[0].parse().unwrap_or(0);

    if let Some(val) = get_value(form, &field) {
        if let Ok(num) = val.parse::<f64>() {
            if num == size as f64 { return None; }
            return Some(format!("The {} must be {}", pretty(field), size));
        }
        if val.len() == size { return None; }
        return Some(format!("The {} must be {} characters", pretty(field), size));
    }
    if let Some(file) = form.files.get(&field) {
        let size_kb = file.content.len() / 1024;
        if size_kb == size { return None; }
        return Some(format!("The {} must be {} kilobytes", pretty(field), size));
    }
    None
}

