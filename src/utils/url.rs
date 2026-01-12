use std::{io::Result};

use url::Url;
use urlencoding::decode;

use crate::utils::{Values, merge};

#[derive(Debug)]
pub struct Domain {
    pub subdomain: String,
    pub domain: String
}


pub fn clean_url(uri: String) -> String {
    if uri == "/" {
        return "".to_string();
    }

    return uri.trim_start_matches("/")
        .trim_end_matches("/")
        .to_string();
}

pub fn uri_to_vec(uri: String) -> Vec<String> {
    return clean_url(uri).split("/").map(|x| x.to_string()).collect();
}

pub fn parse_query_params(query: &str) -> Result<Values> {
    let mut out = Values::new();

    for kv in query.split('&') {
        if kv.is_empty() {
            continue;
        }

        let mut it = kv.splitn(2, '=');
        let k = it.next().unwrap_or("");
        let v = it.next().unwrap_or("");

        out.insert(
            decode(k).unwrap().to_string(), 
            decode(v).unwrap().to_string()
        );
    }

    Ok(out)
} 

pub fn join_paths(one: String, two: String) -> Vec<String> {
    return merge(vec![vec![one], vec![two]]).iter()
        .map(|x| clean_url(x.to_owned()))
        .filter(|x| x != "")
        .collect();
}

pub fn parse_host(host: String) -> Option<Domain> {
    let result = Url::parse(&host);

    if result.is_err() {
        return None;
    }

    let url = result.unwrap();

    if url.host().is_none() {
        return None;
    }

    if !is_domain(url.clone()) {
        return Some(Domain {
            subdomain: String::new(),
            domain: url.host().unwrap().to_string()
        });
    }
    
    let host = url.host_str()?;
    let parts: Vec<String> = host.split(".").map(|v| v.to_string()).collect();

    if parts.len() >= 2 {
        return Some(Domain {
            domain: parts[parts.len()-2..].join("."),
            subdomain: parts[0..parts.len()-2].join("."),
        });
    }

    return Some(Domain {
        domain: parts.join("."),
        subdomain: String::new()
    });
}

pub fn is_domain(url: Url) -> bool {
    match url.host() {
        Some(url::Host::Domain(_)) => true,
        Some(url::Host::Ipv4(_)) => false,
        Some(url::Host::Ipv6(_)) => false,
        None => false,
    }
}