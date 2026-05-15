use std::fmt;

pub enum CloudError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
    Json(serde_json::Error),
    Nix(nix::Error),
    Crypto(String),
}

impl fmt::Display for CloudError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CloudError::Io(e) => write!(f, "IO error: {}", e),
            CloudError::Parse(s) => write!(f, "Parse error: {}", s),
            CloudError::NotFound(s) => write!(f, "Not found: {}", s),
            CloudError::Json(e) => write!(f, "JSON error: {}", e),
            CloudError::Nix(e) => write!(f, "Nix error: {}", e),
            CloudError::Crypto(s) => write!(f, "Crypto error: {}", s),
        }
    }
}

impl fmt::Debug for CloudError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<std::io::Error> for CloudError {
    fn from(e: std::io::Error) -> Self {
        CloudError::Io(e)
    }
}

impl From<serde_json::Error> for CloudError {
    fn from(e: serde_json::Error) -> Self {
        CloudError::Json(e)
    }
}

impl From<nix::Error> for CloudError {
    fn from(e: nix::Error) -> Self {
        CloudError::Nix(e)
    }
}

pub type Result<T> = std::result::Result<T, CloudError>;
