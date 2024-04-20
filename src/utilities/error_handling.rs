use std::fmt;
use std::error::Error;
use std::fmt::{Display, Formatter, write};
use std::process::{ExitCode, Termination};

pub(crate) enum ExitResult {
    Ok,
    Err(ServerError),
}

impl Termination for ExitResult {
    fn report(self) -> ExitCode {
        match self {
            ExitResult::Ok => ExitCode::SUCCESS,
            ExitResult::Err(e) => e.into(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ServerError {
    IoError(std::io::Error),
    ListenerError
}

impl Into<ExitCode> for ServerError {
    fn into(self) -> ExitCode {
        match self {
            ServerError::ListenerError => ExitCode::from(1),
            _ => ExitCode::FAILURE
        }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::IoError(e) => write!(f, "IO error: {e}"), // l10n: ERR_OS_IO(io_err)
            _ => write!(f, "Unknown error!"), // l10n: ERR_UNKNOWN
        }
    }
}

impl Error for ServerError {}
