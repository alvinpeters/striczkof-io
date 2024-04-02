use std::{error, fmt};
use std::error::Error;
use std::fmt::{Display, Formatter, write};
use std::process::{ExitCode, Termination};

pub(crate) enum ExitResult {
    Ok,
    Err(ServerError)
}

#[derive(Debug)]
pub(crate) enum ServerError {
    IoError(std::io::Error),
    ListenerError
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::IoError(e) => write!(f, "IO error: {e}"), // l10n: ERR_OS_IO(io_err)
            _ => write!(f, "Unknown error!"), // l10n: ERR_UNKNOWN
        }
    }
}

impl Termination for ServerError {
    fn report(self) -> ExitCode {
        const DEFAULT_ERR_NUM: u8 = 1;
        let error_number: u8 = match self {
            ServerError::IoError(e) => DEFAULT_ERR_NUM, // TODO: Figure out what to do here
            _ => 1, // Returns exit code 1 as default.
        };
        ExitCode::from(error_number)
    }
}

impl error::Error for ServerError {}
