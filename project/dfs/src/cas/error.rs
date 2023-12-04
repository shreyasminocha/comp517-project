use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct FilesystemError(String);

impl FilesystemError {
    pub fn new(msg: &str) -> Self {
        FilesystemError(msg.to_string())
    }
}

impl Error for FilesystemError {}

impl Display for FilesystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct PathResolutionError(String);

impl PathResolutionError {
    pub fn new(msg: &str) -> Self {
        PathResolutionError(msg.to_string())
    }
}

impl Error for PathResolutionError {}

impl Display for PathResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
