use core::fmt;
use std::path::PathBuf;
use regex;

#[derive(Debug)]
pub enum FsError {
    Io(PathBuf, std::io::Error),
    RegexError(regex::Error),
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::Io(p, err) => {
                write!(f, "{}: {}", p.display(), err)
            }
            FsError::RegexError(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FsError::Io(_, err) => Some(err),
            FsError::RegexError(err) => Some(err),
        }
    }
}

impl From<regex::Error> for FsError {
    fn from(err: regex::Error) -> Self {
        FsError::RegexError(err)
    }
}