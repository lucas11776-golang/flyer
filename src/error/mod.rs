

#[derive(Clone, Default, Debug)]
pub struct Error {
    pub error: String,
    pub message: String,
}

impl Error {
    pub fn new(error: String, message: String) -> Self {
        return Self {
            error: error,
            message: message,
        };
    } 
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for Error {}