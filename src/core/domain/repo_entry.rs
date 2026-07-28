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
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| allowed_exts.contains(ext))
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

pub(crate) fn is_gitignored(path: &Path, patterns: &[Regex]) -> bool {
    let rel_str = path.to_string_lossy();
    patterns.iter().any(|re| re.is_match(&rel_str))
}

#[cfg(test)]
mod tests {
    use crate::core::domain::repo_entry::extract_module_desc;
    use test_case::test_case;

    #[allow(clippy::needless_pass_by_value)]
    #[test_case("// repo-map-desc: desc\nlet a = 1;", Some("desc".to_string()))]
    #[test_case("# repo-map-desc: other desc.\na = 1", Some("other desc.".to_string()))]
    #[test_case("let a = 1;", None)]
    fn test_extract_module_desc(code: &str, expected_result: Option<String>) {
        let res = extract_module_desc(code);
        assert_eq!(res, expected_result);
    }
}
