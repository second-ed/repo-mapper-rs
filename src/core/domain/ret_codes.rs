use std::process::{ExitCode, Termination};

#[derive(Debug, PartialEq, Eq)]
pub enum RetCode {
    NoModification,
    ModifiedReadme,
    FailedParsingFile,
    FailedToWriteReadme,
    InvalidFilename,
}

impl Termination for RetCode {
    fn report(self) -> ExitCode {
        match self {
            RetCode::NoModification | RetCode::ModifiedReadme => ExitCode::SUCCESS,
            RetCode::FailedParsingFile
            | RetCode::FailedToWriteReadme
            | RetCode::InvalidFilename => ExitCode::FAILURE,
        }
    }
}
