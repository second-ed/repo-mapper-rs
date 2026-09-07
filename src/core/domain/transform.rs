// repo-map-desc: Where the file tree is generated

use crate::core::{
    adapters::FileSystem,
    domain::repo_entry::{is_allowed_ext, is_gitignored, is_hidden, is_ignored_dir, RepoEntry},
};
use itertools::Itertools;
use regex::Regex;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::implicit_hasher)]
pub(crate) fn pathbufs_to_repo_entries(
    file_sys: &mut impl FileSystem,
    paths: Vec<PathBuf>,
    root: &Path,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
    dirs_only: bool,
) -> Vec<RepoEntry> {
    paths
        .into_iter()
        .filter(|path| !is_ignored_dir(path, ignore_dirs))
        .filter(|path| {
            if dirs_only {
                true
            } else {
                is_allowed_ext(path, allowed_exts)
            }
        })
        .filter(|path| !is_hidden(path, ignore_hidden))
        .filter(|path| path != root)
        .map(|path| {
            if dirs_only {
                path.parent().map(Path::to_path_buf).unwrap_or_default()
            } else {
                path
            }
        })
        .map(|path| RepoEntry::new(file_sys, root, &path))
        .unique()
        .filter(|e| !e.path().to_str().is_some_and(str::is_empty))
        .filter(|e| !is_gitignored(e.path(), e.is_dir(), gitignored_patterns))
        .collect::<Vec<RepoEntry>>()
}
