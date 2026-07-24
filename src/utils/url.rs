use crate::utils::{Values, env::env};

pub fn url(path: &str) -> String {
    return format!("{}/{}", env("APP_URL").trim_end_matches("/"), path);
}

pub fn clean(path: impl Into<String>) -> Vec<String> {
    return path
        .into()
        .trim_matches('/')
        .split("/")
        .map(|s| String::from(s))
        .filter(|v| v.ne(""))
        .collect::<Vec<String>>();
}

pub fn parse_query(query: impl Into<String>) -> Values {
    return match serde_urlencoded::from_str::<Values>(&query.into()) {
        Ok(values) => values,
        Err(_) => Values::new(),
    };
} 