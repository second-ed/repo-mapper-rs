use regex::Regex;
use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub(crate) struct RepoFile {
    pub path: PathBuf,
}

impl RepoFile {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.path
            .components()
            .map(|p| p.as_os_str().to_str().unwrap_or_default().to_owned())
            .any(|s| s.starts_with('.'))
    }

    pub(crate) fn is_allowed_ext(&self, allowed_exts: &HashSet<String>) -> bool {
        if allowed_exts.is_empty() {
            return true;
        }
        self.path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| allowed_exts.contains(ext))
    }

    pub(crate) fn is_ignored_dir(&self, ignore_dirs: &HashSet<String>) -> bool {
        if ignore_dirs.is_empty() {
            return false;
        }
        self.path
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .filter_map(|path| path.file_name().and_then(OsStr::to_str))
            .any(|name| ignore_dirs.contains(name))
    }

    pub(crate) fn is_gitignored(&self, patterns: &[Regex]) -> bool {
        let rel_str = self.path.to_string_lossy();
        patterns.iter().any(|re| re.is_match(&rel_str))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::domain::{
        repo_file::RepoFile,
        utils::{to_collection_of_type, to_regex_vec, to_str_type},
    };
    use criterion::{black_box, Criterion};
    use std::{collections::HashSet, path::PathBuf};
    use test_case::test_case;

    #[test_case("src/subdir/file.rs", false ; "normal file returns false")]
    #[test_case(".gitignore", true ; "hidden file returns true")]
    #[test_case(".venv/some_dir/some_file.py", true ; "should ignore hidden dir")]
    fn test_repo_file_is_hidden(inp_str: &str, expected_result: bool) {
        let inp_path: PathBuf = to_str_type(inp_str);
        let repo_file = RepoFile::new(inp_path);

        assert_eq!(repo_file.is_hidden(), expected_result);
    }

    #[test_case("src/subdir/file.rs", vec!["rs"], true)]
    #[test_case("src/subdir/file.rs", vec![], true)]
    #[test_case("src/subdir/file.rs", vec!["py"], false)]
    fn test_repo_file_is_allowed_ext(
        inp_str: &str,
        allowed_exts: Vec<&str>,
        expected_result: bool,
    ) {
        let inp_path: PathBuf = to_str_type(inp_str);
        let allowed_exts: HashSet<String> = to_collection_of_type(allowed_exts);
        let repo_file = RepoFile::new(inp_path);

        assert_eq!(repo_file.is_allowed_ext(&allowed_exts), expected_result);
    }

    #[test_case("src/subdir/file.rs", vec!["subdir"], true)]
    #[test_case("src/subdir/file.rs", vec![], false)]
    #[test_case("src/subdir/file.rs", vec!["other_dir"], false)]
    fn test_repo_file_is_ignored_dir(
        inp_str: &str,
        ignored_dirs: Vec<&str>,
        expected_result: bool,
    ) {
        let inp_path: PathBuf = to_str_type(inp_str);
        let ignored_dirs: HashSet<String> = to_collection_of_type(ignored_dirs);
        let repo_file = RepoFile::new(inp_path);

        assert_eq!(repo_file.is_ignored_dir(&ignored_dirs), expected_result);
    }

    #[test_case("src/subdir/file.rs", vec![r"(^|/)target/(.*)?$"], false)]
    #[test_case("some/path/target/subdir/file.rs", vec![ r"(^|/)target/(.*)?$"], true)]
    fn test_repo_file_is_gitignored(
        inp_str: &str,
        regex_patterns: Vec<&str>,
        expected_result: bool,
    ) {
        let inp_path: PathBuf = to_str_type(inp_str);
        let regex_patterns = to_regex_vec(regex_patterns);
        let repo_file = RepoFile::new(inp_path);

        assert_eq!(repo_file.is_gitignored(&regex_patterns), expected_result);
    }

    #[test]
    #[ignore]
    fn new_repo_file() {
        let mut criterion = Criterion::default();
        let path = PathBuf::from("fake/repo/root/src/lib.rs");

        criterion.bench_function("repo_file", |b| {
            b.iter(|| RepoFile::new(black_box(path.clone())))
        });
        criterion.final_summary();
    }
}
