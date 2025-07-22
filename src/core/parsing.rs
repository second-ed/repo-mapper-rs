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

pub struct Args {
    pub scripts_root: PathBuf,
    pub readme_path: PathBuf,
    pub gitignore_path: PathBuf,
    pub allowed_exts: HashSet<String>,
    pub ignore_dirs: HashSet<String>,
    pub ignore_hidden: bool,
}

impl Args {
    pub fn new(
        scripts_root: String,
        readme_path: String,
        gitignore_path: String,
        allowed_exts: Vec<String>,
        ignore_dirs: Vec<String>,
        ignore_hidden: bool,
    ) -> Self {
        let scripts_root = PathBuf::from(scripts_root);
        let readme_path = PathBuf::from(readme_path);
        let gitignore_path = PathBuf::from(gitignore_path);

        let allowed_exts: HashSet<String> = allowed_exts.into_iter().collect();
        let ignore_dirs: HashSet<String> = ignore_dirs.into_iter().collect();

        Self {
            scripts_root,
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

    fn parse(path: impl AsRef<Path>) -> Result<Self, ExitCode> {
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

        match fs::read_to_string(&path) {
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
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, ExitCode> {
        <Self as FileText>::parse(path)
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
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, ExitCode> {
        <Self as FileText>::parse(path)
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
