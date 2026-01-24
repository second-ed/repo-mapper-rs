#[derive(Debug, PartialEq, Eq)]
pub enum RetCode {
    NoModification,
    ModifiedReadme,
    FailedParsingFile,
    FailedToWriteReadme,
    InvalidFilename,
}
