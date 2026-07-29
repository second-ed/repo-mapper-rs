// repo-map-desc: simple implementation of a repo object

use crate::core::adapters::FileSystem;
use regex::Regex;
use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub(crate) enum RepoEntry {
    File { path: PathBuf, desc: Option<String> },
    Dir { path: PathBuf, desc: Option<String> },
}

impl RepoEntry {
    pub(crate) fn new(file_sys: &mut impl FileSystem, root: &Path, path: &Path) -> RepoEntry {
        if path.file_name().and_then(|name| name.to_str()) == Some(".repo-map-desc") {
            let dir_path = path.parent().unwrap_or(Path::new(""));

            let code = file_sys.read_to_string(path).unwrap_or_default();
            let desc = extract_module_desc(&code);

            RepoEntry::Dir {
                path: dir_path
                    .strip_prefix(root)
                    .unwrap_or(dir_path)
                    .to_path_buf(),
                desc,
            }
        } else if file_sys.is_file(path) {
            let code = file_sys.read_to_string(path);
            let desc = extract_module_desc(&code.unwrap_or_default());
            RepoEntry::File {
                path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
                desc,
            }
        } else {
            let desc_file_path = path.join(".repo-map-desc");

            let desc = if file_sys.is_file(&desc_file_path) {
                let code = file_sys.read_to_string(&desc_file_path);
                extract_module_desc(&code.unwrap_or_default())
            } else {
                None
            };
            RepoEntry::Dir {
                path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
                desc,
            }
        }
    }

    pub(crate) fn is_dir(&self) -> bool {
        matches!(self, Self::Dir { .. })
    }

    pub(crate) fn path(&self) -> &Path {
        match self {
            RepoEntry::File { path, .. } | RepoEntry::Dir { path, .. } => path,
        }
    }

    pub(crate) fn desc(&self) -> Option<&str> {
        match self {
            RepoEntry::File { desc, .. } | RepoEntry::Dir { desc, .. } => desc.as_deref(),
        }
    }
}

fn extract_module_desc(code: &str) -> Option<String> {
    code.lines().find_map(|line| {
        let (_, desc) = line.split_once("repo-map-desc:")?;
        Some(desc.trim().to_string())
    })
}

pub(crate) fn is_hidden(path: &Path, ignore_hidden: bool) -> bool {
    if !ignore_hidden {
        return false;
    }
    if path.file_name().and_then(|name| name.to_str()) == Some(".repo-map-desc") {
        return false;
    }
    path.components()
        .map(|p| p.as_os_str().to_str().unwrap_or_default())
        .any(|s| s.starts_with('.'))
}

pub(crate) fn is_allowed_ext(path: &Path, allowed_exts: &HashSet<String>) -> bool {
    if allowed_exts.is_empty() {
        return true;
    }
    if path.file_name().and_then(|name| name.to_str()) == Some(".repo-map-desc") {
        return true;
    }
    match path.extension() {
        Some(s) => s.to_str().is_some_and(|ext| allowed_exts.contains(ext)),
        None => true,
    }
}

pub(crate) fn is_ignored_dir(path: &Path, ignore_dirs: &HashSet<String>) -> bool {
    if ignore_dirs.is_empty() {
        return false;
    }
    path.parent()
        .into_iter()
        .flat_map(Path::ancestors)
        .filter_map(|path| path.file_name().and_then(OsStr::to_str))
        .any(|name| ignore_dirs.contains(name))
}

pub(crate) fn is_gitignored(path: &Path, is_dir: bool, patterns: &[Regex]) -> bool {
    let mut rel_str = path.to_string_lossy().into_owned();
    if is_dir {
        rel_str.push('/');
    }
    patterns.iter().any(|re| re.is_match(&rel_str))
}

#[cfg(test)]
mod tests {
    use crate::core::domain::{
        repo_entry::{
            extract_module_desc, is_allowed_ext, is_gitignored, is_hidden, is_ignored_dir,
        },
        utils::{to_collection_of_type, to_regex_vec},
    };
    use regex::Regex;
    use std::{collections::HashSet, path::Path};
    use test_case::test_case;

    #[allow(clippy::needless_pass_by_value)]
    #[test_case("// repo-map-desc: desc\nlet a = 1;", Some("desc".to_string()))]
    #[test_case("# repo-map-desc: other desc.\na = 1", Some("other desc.".to_string()))]
    #[test_case("let a = 1;", None)]
    fn test_extract_module_desc(code: &str, expected_result: Option<String>) {
        let res = extract_module_desc(code);
        assert_eq!(res, expected_result);
    }

    #[test_case("some/public/path.rs", true, false ; "given public path when ignore hidden is true then returns false")]
    #[test_case("some/public/path.rs", false, false ; "given public path when ignore hidden is false then returns false")]
    #[test_case("some/hidden/.file.py", true, true ; "given hidden file when ignore hidden is true then returns true")]
    #[test_case("some/hidden/.file.py", false, false ; "given hidden file when ignore hidden is false then returns false")]
    #[test_case("some/hidden/.dir/nested.py", true, true ; "given hidden dir when ignore hidden is true then returns true")]
    #[test_case("some/hidden/.dir/nested.py", false, false ; "given hidden dir when ignore hidden is false then returns false")]
    fn test_is_hidden(inp_path: &str, ignore_hidden: bool, expected_result: bool) {
        let res = is_hidden(Path::new(inp_path), ignore_hidden);
        assert_eq!(res, expected_result);
    }

    #[test_case("some/public/path.rs", vec!["rs", "py"], true ; "given a rs source file, when called with allowed exts including rs, then returns true")]
    #[test_case("some/public/path.rs", vec!["py"], false ; "given a rs source file, when called with allowed exts excluding rs, then returns false")]
    #[test_case("some/public/.repo-map-desc", vec!["py"], true ; "given a .repo-map-desc, when called with allowed exts, then returns true")]
    #[test_case("some/public/.repo-map-desc", vec![], true ; "given a .repo-map-desc, when called without allowed exts, then returns true")]
    #[test_case("some/public/path/no/ext", vec!["py"], true ; "given a source file with no ext, when called with allowed exts, then returns true")]
    fn test_is_allowed_ext(inp_path: &str, inp_allowed_exts: Vec<&str>, expected_result: bool) {
        let allowed_exts: HashSet<String> = to_collection_of_type(inp_allowed_exts);
        let res = is_allowed_ext(Path::new(inp_path), &allowed_exts);
        assert_eq!(res, expected_result);
    }

    #[test_case("some/public/path.rs", vec!["other_dir"], false ; "given a public path, when the ignored_dirs vec does not include it, then returns false")]
    #[test_case("some/ignored/path.rs", vec!["ignored"], true ; "given an ignored path, when the ignored_dirs vec does include it, then returns true")]
    #[test_case("some/really/nested/ignored/deep/path.rs", vec!["ignored"], true ; "given a nested ignored path, when the ignored_dirs vec does include it, then returns true")]
    #[test_case("some/random/path.rs", vec![], false ; "given any path, when the ignored_dirs vec is empty, then returns false")]
    fn test_is_ignored_dir(inp_path: &str, inp_ignored_dirs: Vec<&str>, expected_result: bool) {
        let ignored_dirs: HashSet<String> = to_collection_of_type(inp_ignored_dirs);
        let res = is_ignored_dir(Path::new(inp_path), &ignored_dirs);
        assert_eq!(res, expected_result);
    }

    #[test_case("some/dir/__pycache__", true, vec!["(^|/)__pycache__/(.*)?$"], true ; "given a path and is_dir is true, when patterns includes '__pycache__/', then returns true")]
    #[test_case("some/dir/file.log", false, vec!["(^|/)[^/]*\\.log$"], true ; "given a path that ends with .log and is_dir is false, when pattens includes '*.log', then returns true")]
    #[test_case("some/dir/file.rs", false, vec!["(^|/)[^/]*\\.log$"], false ; "given a path that does not end with .log and is_dir is false, when pattens includes '*.log', then returns false")]
    #[test_case("some/dir/scratch.py", false, vec!["(^|/)scratch[^/]*\\.py$"], true ; "given a path that exactly matches a pattern, when the function is called, then returns true" )]
    #[test_case("some/dir/scratch_test.py", false, vec!["(^|/)scratch[^/]*\\.py$"], true ; "given a path that starts with a pattern, when the function is called, then returns true" )]
    #[test_case("some/dir/scratch.rs", false, vec!["(^|/)scratch[^/]*\\.py$"], false ; "given a path does not match the pattern, when the function is called, then returns false" )]
    fn test_is_gitignored(
        inp_path: &str,
        is_dir: bool,
        inp_gitignore_patterns: Vec<&str>,
        expected_result: bool,
    ) {
        let patterns: Vec<Regex> = to_regex_vec(inp_gitignore_patterns);
        let res = is_gitignored(Path::new(inp_path), is_dir, &patterns);
        assert_eq!(res, expected_result);
    }
}
