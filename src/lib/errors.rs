use std::{error, fmt, io};

#[derive(Debug)]
pub enum EsbuildConfigError {
    ConfigParseError,
    ConfigPathError,
    Io(io::Error),
}
impl fmt::Display for EsbuildConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An unknown error happened.")
    }
}

#[derive(Debug)]
pub enum ConfigPathError {
    Io(io::Error),
}
impl fmt::Display for ConfigPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Couldn’t find or open the esbuild configuration file.")
    }
}

#[derive(Debug)]
pub enum ConfigParseError {
    InvalidConfigError,
    JsonError(json::Error),
}
impl fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid esbuild configuration format.")
    }
}

#[derive(Debug)]
pub struct InvalidConfigError;
impl error::Error for InvalidConfigError {}
impl fmt::Display for InvalidConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid esbuild configuration format.")
    }
}
