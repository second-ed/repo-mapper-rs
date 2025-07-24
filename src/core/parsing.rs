use crate::core::{adapters::FileSystem, converters::to_hashset};
use colored::Colorize;
use regex::Regex;
use std::{
    collections::HashSet,
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
    process::ExitCode,
};
use walkdir::WalkDir;

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub repo_root: PathBuf,
    pub readme_path: PathBuf,
    pub gitignore_path: PathBuf,
    pub allowed_exts: HashSet<String>,
    pub ignore_dirs: HashSet<String>,
    pub ignore_hidden: bool,
}

impl Args {
    pub fn new(
        repo_root: String,
        readme_path: String,
        gitignore_path: String,
        allowed_exts: Vec<String>,
        ignore_dirs: Vec<String>,
        ignore_hidden: bool,
    ) -> Self {
        let repo_root = PathBuf::from(repo_root);
        let readme_path = PathBuf::from(readme_path);
        let gitignore_path = PathBuf::from(gitignore_path);

        let allowed_exts: HashSet<String> = to_hashset(allowed_exts);
        let ignore_dirs: HashSet<String> = to_hashset(ignore_dirs);

        Self {
            repo_root,
            readme_path,
            gitignore_path,
            allowed_exts,
            ignore_dirs,
            ignore_hidden,
        }
    }
}

trait FileText: Sized {
    const EXPECTED_FILENAME: &'static str;

    fn from_string(s: String) -> Self;

    fn parse(file_sys: &mut impl FileSystem, path: impl AsRef<Path>) -> Result<Self, ExitCode> {
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
            return Err(ExitCode::FAILURE);
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
                Err(ExitCode::FAILURE)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct GitIgnore(String);

impl FileText for GitIgnore {
    const EXPECTED_FILENAME: &'static str = ".gitignore";

    fn from_string(s: String) -> Self {
        GitIgnore(s)
    }
}

impl GitIgnore {
    pub fn parse(file_sys: &mut impl FileSystem, path: impl AsRef<Path>) -> Result<Self, ExitCode> {
        <Self as FileText>::parse(file_sys, path)
    }

    pub fn parse_lines(&self) -> Vec<Regex> {
        self.0
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|pattern| {
                let mut regex_str = String::new();

                regex_str.push_str("(^|/)");

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

impl Deref for GitIgnore {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReadMe(String);

impl FileText for ReadMe {
    const EXPECTED_FILENAME: &'static str = "README.md";

    fn from_string(s: String) -> Self {
        ReadMe(s)
    }
}

impl ReadMe {
    pub fn parse(file_sys: &mut impl FileSystem, path: impl AsRef<Path>) -> Result<Self, ExitCode> {
        <Self as FileText>::parse(file_sys, path)
    }

    pub fn write(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(path, &self.0)
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

impl Deref for ReadMe {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn list_files(path: impl AsRef<Path>) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Args, GitIgnore, ReadMe};
    use crate::core::converters::{to_hashset, to_regex_vec};
    use regex::Regex;
    use std::path::PathBuf;
    use test_case::test_case;

    #[test]
    fn test_args() {
        let args = Args::new(
            "root".to_string(),
            "readme.md".to_string(),
            ".gitignore".to_string(),
            vec!["py".to_string(), "rs".to_string()],
            vec![],
            true,
        );

        let expected_result = Args {
            repo_root: PathBuf::from("root"),
            readme_path: PathBuf::from("readme.md"),
            gitignore_path: PathBuf::from(".gitignore"),
            allowed_exts: to_hashset(vec!["py", "rs"]),
            ignore_dirs: to_hashset(Vec::<&str>::new()),
            ignore_hidden: true,
        };

        assert_eq!(args, expected_result);
    }

    #[test]
    fn test_gitignore() {
        fn regex_vec_to_strs(vec: &[Regex]) -> Vec<&str> {
            vec.iter().map(|re| re.as_str()).collect()
        }

        let gitignore = GitIgnore(".pytest_cache/\n*.log\n?scratch.py".to_string());

        let actual_result = gitignore.parse_lines();
        let expected_result = to_regex_vec(vec![
            "(^|/)\\.pytest_cache/(.*)?$",
            "(^|/)[^/]*\\.log$",
            "(^|/).scratch\\.py$",
        ]);

        assert_eq!(
            regex_vec_to_strs(&actual_result),
            regex_vec_to_strs(&expected_result)
        );
    }

    #[test_case(
        "#Some readme", 
        "appended", 
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
