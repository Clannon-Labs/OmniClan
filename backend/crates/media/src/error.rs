
use super::workers::error::WorkerError;

#[derive(Debug)]
pub(crate) enum MediaError {
    Io(std::io::Error),
    Worker(WorkerError),
}

// Just incase std::io::Error slips out from internal
// functions by mistake
impl From<std::io::Error> for MediaError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<WorkerError> for MediaError {
    fn from(err: WorkerError) -> Self {
        Self::Worker(err)
    }
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Media I/O error: {err}"),
            Self::Worker(err) => write!(f, "Worker error: {err}"),
        }
    }
}

impl std::error::Error for MediaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Worker(err) => Some(err),
        }
    }
}
