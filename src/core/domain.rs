use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    ffi,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FileTree {
    pub nodes: HashMap<String, FileTree>,
}

impl FileTree {
    pub fn new() -> Self {
        FileTree {
            nodes: HashMap::new(),
        }
    }

    fn insert(&mut self, path: &Path) {
        let parts = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string());

        let mut node = self;
        for part in parts {
            node = node.nodes.entry(part).or_default();
        }
    }

    pub fn create_map(mut self, paths: Vec<PathBuf>) -> Self {
        for path in paths {
            self.insert(&path);
        }
        self
    }

    pub fn render(&self) -> String {
        fn _walk(tree: &HashMap<String, FileTree>, prefix: String, out: &mut Vec<String>) {
            let mut items: Vec<_> = tree.iter().collect();

            items.sort_by_key(|(name, node)| (node.nodes.is_empty(), name.to_owned()));

            for (i, (name, _)) in items.iter().enumerate() {
                let is_last = i == items.len() - 1;
                let connector = if is_last { "└── " } else { "├── " };
                out.push(format!("{prefix}{connector}{name}"));

                if let Some(subtree) = tree.get(*name) {
                    let new_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
                    _walk(&subtree.nodes, new_prefix, out);
                }
            }
        }

        let mut out = Vec::new();
        _walk(&self.nodes, String::new(), &mut out);
        format!("# Repo map\n```\n{}\n::\n```", out.join("\n"))
    }
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}

pub fn filter_paths(
    paths: Vec<PathBuf>,
    root: &PathBuf,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
) -> Vec<PathBuf> {
    #[inline(always)]
    fn _is_hidden(path: &Path) -> bool {
        path.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    #[inline(always)]
    fn _is_allowed_ext(path: &Path, allowed_exts: &HashSet<String>) -> bool {
        if allowed_exts.is_empty() {
            return true;
        }
        path.extension()
            .and_then(ffi::OsStr::to_str)
            .map(|ext| allowed_exts.contains(ext))
            .unwrap_or(false)
    }

    #[inline(always)]
    fn _is_ignored_dir(path: &Path, root: &PathBuf, ignore_dirs: &HashSet<String>) -> bool {
        if ignore_dirs.is_empty() {
            return true;
        }
        match path.strip_prefix(root) {
            Ok(stripped) => stripped.ancestors().any(|anc| {
                anc.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| ignore_dirs.contains(name))
                    .unwrap_or(false)
            }),
            Err(_) => true,
        }
    }

    #[inline(always)]
    fn _is_gitignored(path: &Path, patterns: &[Regex]) -> bool {
        let rel_str = path.to_string_lossy();
        patterns.iter().any(|re| re.is_match(&rel_str))
    }

    paths
        .into_par_iter()
        .filter(|e| !ignore_hidden || !_is_hidden(e))
        .filter(|e| _is_allowed_ext(e, allowed_exts))
        .filter(|e| !_is_ignored_dir(e, root, ignore_dirs))
        .filter_map(|e| e.as_path().strip_prefix(root).ok().map(|p| p.to_owned()))
        .filter(|p| !_is_gitignored(p, gitignored_patterns))
        .collect()
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use std::{collections::HashSet, path::PathBuf};

    use super::filter_paths;

    fn get_mock_repo_vec() -> Vec<&'static str> {
        vec![
            "user/root/repo/.github/workflows/ci.yaml",
            "user/root/repo/src/core/some_file.rs",
            "user/root/repo/src/core/some_file2.rs",
            "user/root/repo/python/cli/__init__.py",
            "user/root/repo/.venv/site-packages/__init__.py",
            "user/root/repo/README.md",
            "user/root/repo/scrap/some_file.rs",
            "user/root/repo/.pytest_cache/some_cache.py",
            "user/root/repo/target/some_build.rs",
            "user/root/repo/Cargo.toml",
            "user/root/repo/.hidden.toml",
        ]
    }

    fn create_pathbuf_vec(inp_vec: Vec<&str>) -> Vec<PathBuf> {
        inp_vec.iter().map(PathBuf::from).collect()
    }

    fn create_hashset(inp_vec: Vec<&str>) -> HashSet<String> {
        inp_vec
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>()
    }

    fn create_gitignore_patterns(inp_vec: Vec<&str>) -> Vec<Regex> {
        inp_vec
            .into_iter()
            .filter_map(|s| Regex::new(s).ok())
            .collect::<Vec<Regex>>()
    }

    #[test]
    fn test_filter_paths_ignore_hidden() {
        let paths = create_pathbuf_vec(get_mock_repo_vec());

        let root = PathBuf::from("user/root/repo");
        let allowed_exts = create_hashset(vec!["py", "rs", "toml"]);
        let ignore_dirs = create_hashset(vec!["scrap", ".venv"]);
        let gitignored_patterns =
            create_gitignore_patterns(vec![r"(^|/)\.pytest_cache/(.*)?$", r"(^|/)target/(.*)?$"]);

        let expected_result: Vec<PathBuf> = create_pathbuf_vec(vec![
            "src/core/some_file.rs",
            "src/core/some_file2.rs",
            "python/cli/__init__.py",
            "Cargo.toml",
        ]);

        let actual_result = filter_paths(
            paths,
            &root,
            &allowed_exts,
            &ignore_dirs,
            &gitignored_patterns,
            true,
        );

        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn test_filter_paths_process_hidden() {
        let paths = create_pathbuf_vec(get_mock_repo_vec());

        let root = PathBuf::from("user/root/repo");
        let allowed_exts = create_hashset(vec!["rs", "toml"]);
        let ignore_dirs = create_hashset(vec!["scrap", ".venv"]);
        let gitignored_patterns =
            create_gitignore_patterns(vec![r"(^|/)\.pytest_cache/(.*)?$", r"(^|/)target/(.*)?$"]);

        let expected_result: Vec<PathBuf> = create_pathbuf_vec(vec![
            "src/core/some_file.rs",
            "src/core/some_file2.rs",
            "Cargo.toml",
            ".hidden.toml",
        ]);

        let actual_result = filter_paths(
            paths,
            &root,
            &allowed_exts,
            &ignore_dirs,
            &gitignored_patterns,
            false,
        );

        assert_eq!(actual_result, expected_result);
    }
}
