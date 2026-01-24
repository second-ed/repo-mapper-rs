use crate::core::{adapters::FileSystem, domain::ret_codes::RetCode};
use colored::Colorize;
use std::path::Path;

pub(crate) trait FileText: Sized {
    const EXPECTED_FILENAME: &'static str;

    fn from_string(s: String) -> Self;

    fn parse(file_sys: &mut impl FileSystem, path: impl AsRef<Path>) -> Result<Self, RetCode> {
        let path = path.as_ref().to_path_buf();
        let basename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Invalid basename");

        if basename != Self::EXPECTED_FILENAME {
            eprintln!(
                "{}",
                format!(
                    "Invalid `{}` basename: `{}`",
                    Self::EXPECTED_FILENAME,
                    basename
                )
                .red()
                .bold(),
            );
            return Err(RetCode::InvalidFilename);
        }

        match file_sys.read_to_string(&path) {
            Ok(contents) => Ok(Self::from_string(contents)),
            Err(e) => {
                eprintln!(
                    "{} {}",
                    format!("Failed to parse `{}`", Self::EXPECTED_FILENAME)
                        .red()
                        .bold(),
                    e
                );
                Err(RetCode::FailedParsingFile)
            }
        }
    }
}
