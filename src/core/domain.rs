use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    ffi,
    path::{Path, PathBuf},
};
use walkdir::DirEntry;

use crate::core::parsing::ReadMe;

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
            node = node.nodes.entry(part).or_insert_with(FileTree::new);
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

pub fn filter_entries(
    entries: Vec<DirEntry>,
    root: &PathBuf,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
) -> Vec<PathBuf> {
    #[inline(always)]
    fn _is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    #[inline(always)]
    fn _is_allowed_ext(entry: &DirEntry, allowed_exts: &HashSet<String>) -> bool {
        if allowed_exts.is_empty() {
            return true;
        }
        entry
            .path()
            .extension()
            .and_then(ffi::OsStr::to_str)
            .map(|ext| allowed_exts.contains(ext))
            .unwrap_or(false)
    }

    #[inline(always)]
    fn _is_ignored_dir(entry: &DirEntry, root: &PathBuf, ignore_dirs: &HashSet<String>) -> bool {
        if ignore_dirs.is_empty() {
            return true;
        }
        match entry.path().strip_prefix(root) {
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

    entries
        .into_par_iter()
        .filter(|e| !ignore_hidden || !_is_hidden(e))
        .filter(|e| _is_allowed_ext(e, allowed_exts))
        .filter(|e| !_is_ignored_dir(e, root, ignore_dirs))
        .filter_map(|e| e.path().strip_prefix(root).ok().map(|p| p.to_owned()))
        .filter(|p| !_is_gitignored(p, gitignored_patterns))
        .collect()
}

pub fn update_readme(readme: &ReadMe, repo_map: String) -> ReadMe {
    let pattern = Regex::new(r"(?s)(?m)^# Repo map\n```.*?^::\n```").expect("valid regex");

    let updated = if pattern.is_match(&readme.0) {
        pattern.replace(&readme.0, repo_map).into_owned()
    } else {
        format!("{}\n\n{}", readme.0, repo_map)
    };
    ReadMe(updated)
}
