use regex::Regex;
use std::{collections::HashSet, ffi::OsStr, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct RepoFile {
    pub path: PathBuf,
}

impl RepoFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn is_hidden(&self) -> bool {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    pub fn is_allowed_ext(&self, allowed_exts: &HashSet<String>) -> bool {
        if allowed_exts.is_empty() {
            return true;
        }
        os_str_contains(self.path.extension(), allowed_exts)
    }

    pub fn is_ignored_dir(&self, ignore_dirs: &HashSet<String>) -> bool {
        if ignore_dirs.is_empty() {
            return false;
        }
        self.path
            .ancestors()
            .any(|anc| os_str_contains(anc.file_name(), ignore_dirs))
    }

    pub fn is_gitignored(&self, patterns: &[Regex]) -> bool {
        let rel_str = self.path.to_string_lossy();
        patterns.iter().any(|re| re.is_match(&rel_str))
    }
}

fn os_str_contains(os_str: Option<&OsStr>, collection: &HashSet<String>) -> bool {
    os_str
        .and_then(|s| s.to_str())
        .map(|ext| collection.contains(ext))
        .unwrap_or(false)
}
