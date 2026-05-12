use crate::utils::{Values, merge};

pub fn clean_url(uri: String) -> String {
    if uri == "/" {
        return "".to_string();
    }

    return uri.trim_start_matches("/")
        .trim_end_matches("/")
        .to_string();
}

pub fn uri_to_segments(uri: String) -> Vec<String> {
    return uri.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| String::from(s))
        .collect()
}

pub fn parse_query_params(query: &str) -> Values {
    return match serde_urlencoded::from_str::<Values>(query) {
        Ok(values) => values,
        Err(_) => Values::new(),
    };
} 

pub fn join_paths(one: String, two: String) -> Vec<String> {
    return merge(vec![vec![one], vec![two]]).iter()
        .map(|x| clean_url(x.to_owned()))
        .filter(|x| x != "")
        .collect();
}

pub fn join_url(url: Vec<String>) -> String {
    return url.iter()
        .map(|u| String::from(u.trim_matches('/'))).collect::<Vec<_>>()
        .join("/")
        .trim_matches('/')
        .to_string();
}

pub fn url(path: &str) -> String {
    return format!("{}/{}", crate::utils::env("APP_URL").trim_end_matches("/"), path);
}