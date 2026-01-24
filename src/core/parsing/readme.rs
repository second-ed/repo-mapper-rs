use crate::core::{adapters::FileSystem, domain::ret_codes::RetCode, parsing::file_text::FileText};
use regex::Regex;
use std::{io, path::Path};

#[derive(Debug, Eq, PartialEq)]
pub struct ReadMe(String);

impl FileText for ReadMe {
    const EXPECTED_FILENAME: &'static str = "README.md";

    fn from_string(s: String) -> Self {
        ReadMe(s)
    }
}

impl ReadMe {
    pub fn parse(file_sys: &mut impl FileSystem, path: impl AsRef<Path>) -> Result<Self, RetCode> {
        <Self as FileText>::parse(file_sys, path)
    }

    pub fn write(&self, file_sys: &mut impl FileSystem, path: &Path) -> Result<(), io::Error> {
        file_sys.write(path, &self.0)
    }

    pub fn update_readme(&self, repo_map: String) -> ReadMe {
        let pattern = Regex::new(r"(?s)(?m)^# Repo map\n```.*?^::\n```").expect("valid regex");

        let updated = if pattern.is_match(&self.0) {
            pattern.replace(&self.0, repo_map).into_owned()
        } else {
            format!("{}\n\n{}", self.0, repo_map)
        };
        ReadMe(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::ReadMe;
    use test_case::test_case;

    #[test_case(
        "#Some readme", "appended",
        "#Some readme\n\nappended" ;
        "Ensure appends if the repo map doesn't exist"
    )]
    #[test_case(
        "#Some readme\n# Repo map\n```\noriginal\n::\n```\n#Some line afterwards",
        "# Repo map\n```\nmodified\n::\n```",
        "#Some readme\n# Repo map\n```\nmodified\n::\n```\n#Some line afterwards" ;
        "Ensure replaces if the repo map exists"
    )]
    fn test_readme_if_not_already_exists(inp_readme: &str, repo_map: &str, expected_result: &str) {
        let readme = ReadMe(inp_readme.into());
        assert_eq!(
            readme.update_readme(repo_map.into()),
            ReadMe(expected_result.into()),
        );
    }
}
