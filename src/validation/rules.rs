use crate::request::form::Form;

fn pretty(value: String) -> String {
    let temp: Vec<&str> = value.split("_").collect();
    return temp.join(" ");
}

pub async fn required(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_some() {
        return None
    }

    if form.values.get(field.as_str()).is_some() && form.values.get(field.as_str()).unwrap() != "" {
        return None
    }

    return Some(format!("The {} is required", pretty(field)))
}

pub async fn string(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.values.get(field.as_str()).is_none() {
        return Some(format!("The {} must be a string", pretty(field)))
    }

    return None
}

pub async fn min(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_some() {
        if form.files.get(field.as_str()).unwrap().content.len() < args[0].parse().unwrap() {
            return Some(format!("The {} must have minimum of {} kilobytes", pretty(field), args[0]));
        }

        return None
    }

    if form.values.get(field.as_str()).is_some() && form.values.get(field.as_str()).unwrap().len() >= args[0].parse().unwrap() {
        return None 
    }

    return Some(format!("The {} must have minimum of {} characters", pretty(field), args[0]));
}

pub async fn max(form: &Form, field: String, args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_some() {
        if form.files.get(field.as_str()).unwrap().content.len() > args[0].parse().unwrap() {
            return Some(format!("The {} must have minimum of {} kilobytes", pretty(field), args[0]));
        }

        return None
    }

    if form.values.get(field.as_str()).is_some() && form.values.get(field.as_str()).unwrap().len() <= args[0].parse().unwrap() {
        return None 
    }

    return Some(format!("The {} must have minimum of {} characters", pretty(field), args[0]));
}

pub async fn confirmed(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if let Some(confirm) = form.values.get(&format!("{}_confirmed", field)) && confirm.eq(form.values.get(&field).unwrap()) {
        return None 
    }

    return Some(format!("The {} does not match {} confirmation", pretty(field.clone()), pretty(field)));
}

pub async fn file(form: &Form, field: String, _args: Vec<String>) -> Option<String> {
    if form.files.get(field.as_str()).is_none() {
        return Some(format!("The {} must be type of file", pretty(field)))
    }
    
    return None
}