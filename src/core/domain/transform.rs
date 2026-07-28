// repo-map-desc: Where the file tree is generated

use crate::core::{
    adapters::FileSystem,
    domain::{
        file_tree::FileTree,
        repo_entry::{is_allowed_ext, is_gitignored, is_hidden, is_ignored_dir, RepoEntry},
    },
};
use itertools::Itertools;
use regex::Regex;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::implicit_hasher)]
pub fn pathbufs_to_filetree(
    file_sys: &mut impl FileSystem,
    paths: Vec<PathBuf>,
    root: &Path,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
    dirs_only: bool,
) -> FileTree {
    let repo_files = paths
        .into_iter()
        .filter(|path| !is_ignored_dir(path, ignore_dirs))
        .filter(|path| is_allowed_ext(path, allowed_exts))
        .filter(|path| !is_gitignored(path, gitignored_patterns))
        .filter(|path| !is_hidden(path, ignore_hidden))
        .map(|path| {
            if dirs_only {
                path.parent().map(Path::to_path_buf).unwrap_or_default()
            } else {
                path
            }
        })
        .unique_by(PathBuf::clone)
        .map(|path| RepoEntry::new(file_sys, root, &path))
        .filter(|e| !e.path().to_str().is_some_and(str::is_empty))
        .collect::<Vec<RepoEntry>>();

    FileTree::from_repo_entries(&repo_files)
}
