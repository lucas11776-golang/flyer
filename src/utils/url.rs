use crate::request::Values;

pub fn clean_url(uri: String) -> String {
    return uri.trim_start_matches("/")
        .trim_end_matches("/")
        .to_string();
}

pub fn clean_uri_to_vec(uri: String) -> Vec<String> {
    return clean_url(uri).split("/").map(|x| x.to_string()).collect();
}

pub fn parse_query_params(query: &str) -> Values {
    let mut out = Values::new();
    for kv in query.split('&') {
        if kv.is_empty() {
            continue;
        }
        let mut it = kv.splitn(2, '=');
        let k = it.next().unwrap_or("").to_string();
        let v = it.next().unwrap_or("").to_string();
        out.insert(k, v);
    }
    out
}