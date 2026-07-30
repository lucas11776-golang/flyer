pub fn load(path: &str) {
    dotenv::from_filename(path).unwrap();
}

pub fn env(key: impl Into<String>) -> String {
    return std::env::var(key.into()).unwrap_or(String::new());
} 