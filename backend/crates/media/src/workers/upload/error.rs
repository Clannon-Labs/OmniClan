
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum UploadError {
    Io{
        source: std::io::Error,
        path: Option<PathBuf>,
    },
    TooLarge {
        limit: u64,
    },
    InvalidMedia {
        reason: String,
    },
    InvalidState {
        reason: String,
    },
    Write{
        source: std::io::Error,
        path: PathBuf,
    }
}

impl From<std::io::Error> for UploadError {
    fn from(err: std::io::Error) -> Self {
        UploadError::Io {
            source: err,
            path: None,
        }
    }
}

impl UploadError {
    pub(crate) fn io(error: std::io::Error, path: impl Into<PathBuf>) -> Self {
        Self::Io {
            source: error,
            path: Some(path.into())
        }
    }
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io {source, path} => {
                match path {
                    Some(path) => {
                        write!(f, "I/O error while accessing {}: {source}!", path.display())
                    },
                    None => {
                        write!(f, "I/O error: {source}")
                    }
                }
            },
            Self::TooLarge {limit} => {
                write!(f, "Upload exceeded the size limit of {limit} bytes!")
            },
            Self::InvalidMedia{reason} => {
                write!(f, "Invalid Media: {reason}")
            },
            Self::InvalidState{reason} => {
                write!(f, "Invalid State: {reason}")
            },
            Self::Write{source, path} => {
                write!(f, "Write error: {source} in {path:?}")
            }
        }
    }
}

impl std::error::Error for UploadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io {source, ..} => Some(source),
            Self::TooLarge{..} => None,
            Self::InvalidState {..} => None,
            Self::InvalidMedia {..} => None,
            Self::Write{source, ..} => Some(source),
        }
    }
}
