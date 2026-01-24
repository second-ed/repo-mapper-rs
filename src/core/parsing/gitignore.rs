use crate::core::{adapters::FileSystem, domain::ret_codes::RetCode, parsing::file_text::FileText};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub struct GitIgnore(String);

impl FileText for GitIgnore {
    const EXPECTED_FILENAME: &'static str = ".gitignore";

    fn from_string(s: String) -> Self {
        GitIgnore(s)
    }
}

impl GitIgnore {
    pub fn parse(file_sys: &mut impl FileSystem, path: impl AsRef<Path>) -> Result<Self, RetCode> {
        <Self as FileText>::parse(file_sys, path)
    }

    pub fn parse_lines(&self) -> Vec<Regex> {
        self.0
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|pattern| {
                let mut regex_str = String::new();

                regex_str.push_str("(^|/)");
                let pattern = pattern.trim_start_matches("/");

                for c in pattern.chars() {
                    match c {
                        '*' => regex_str.push_str("[^/]*"),
                        '?' => regex_str.push('.'),
                        '.' => regex_str.push_str(r"\."),
                        _ => regex_str.push(c),
                    }
                }

                if pattern.ends_with('/') {
                    regex_str.push_str("(.*)?$");
                } else {
                    regex_str.push('$');
                }

                Regex::new(&regex_str).ok()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::GitIgnore;
    use crate::core::domain::utils::to_regex_vec;
    use regex::Regex;

    #[test]
    fn test_gitignore() {
        fn regex_vec_to_strs(vec: &[Regex]) -> Vec<&str> {
            vec.iter().map(|re| re.as_str()).collect()
        }

        let gitignore = GitIgnore(".pytest_cache/\n*.log\n?scratch.py\n/outputs/".to_string());

        let actual_result = gitignore.parse_lines();
        let expected_result = to_regex_vec(vec![
            "(^|/)\\.pytest_cache/(.*)?$",
            "(^|/)[^/]*\\.log$",
            "(^|/).scratch\\.py$",
            "(^|/)outputs/(.*)?$",
        ]);

        assert_eq!(
            regex_vec_to_strs(&actual_result),
            regex_vec_to_strs(&expected_result)
        );
    }
}
