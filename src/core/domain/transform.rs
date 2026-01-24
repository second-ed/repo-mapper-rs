// repo-map-desc: Where the file tree is generated

use crate::core::{
    adapters::FileSystem,
    domain::{file_node::FileNode, file_tree::FileTree, repo_file::RepoFile},
};
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[allow(clippy::too_many_arguments)]
pub fn pathbufs_to_filetree(
    file_sys: &mut impl FileSystem,
    paths: Vec<PathBuf>,
    root: &PathBuf,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
    dirs_only: bool,
) -> FileTree {
    let repo_files = pathbufs_to_repo_files(paths);
    let repo_files = filter_repo_files(
        repo_files,
        allowed_exts,
        ignore_dirs,
        gitignored_patterns,
        ignore_hidden,
    );

    let repo_files = if dirs_only {
        filter_dirnames(repo_files.clone())
    } else {
        repo_files
    };

    let file_nodes = repo_file_to_file_node(file_sys, repo_files, root);
    FileTree::from_file_nodes(&file_nodes)
}

fn filter_dirnames(repo_files: Vec<RepoFile>) -> Vec<RepoFile> {
    repo_files
        .into_iter()
        .map(|repo_file| {
            let parent_path = repo_file
                .path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default();
            RepoFile::new(parent_path)
        })
        .dedup()
        .collect()
}

fn pathbufs_to_repo_files(paths: Vec<PathBuf>) -> Vec<RepoFile> {
    paths.into_par_iter().map(RepoFile::new).collect()
}

fn filter_repo_files(
    repo_files: Vec<RepoFile>,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
) -> Vec<RepoFile> {
    repo_files
        .into_par_iter()
        .filter(|file| {
            (!ignore_hidden || !file.is_hidden())
                & file.is_allowed_ext(allowed_exts)
                & !file.is_ignored_dir(ignore_dirs)
                & !file.is_gitignored(gitignored_patterns)
        })
        .collect()
}

fn repo_file_to_file_node(
    file_sys: &mut impl FileSystem,
    repo_files: Vec<RepoFile>,
    root: &PathBuf,
) -> Vec<FileNode> {
    repo_files
        .into_iter()
        .map(|repo_file| {
            let desc = if repo_file.path.is_file() {
                let code = file_sys.read_to_string(&repo_file.path);
                extract_module_desc(&code.unwrap_or_default())
            } else {
                None
            };

            FileNode::from_repo_file(repo_file, root, desc)
        })
        .collect()
}

fn extract_module_desc(code: &str) -> Option<String> {
    code.lines().find_map(|line| {
        let (_, desc) = line.split_once("repo-map-desc:")?;
        Some(desc.trim().to_string())
    })
}

#[cfg(test)]
mod tests {
    use crate::core::domain::transform::extract_module_desc;
    use test_case::test_case;

    #[test_case("// repo-map-desc: desc\nlet a = 1;", Some("desc".to_string()))]
    #[test_case("# repo-map-desc: other desc.\na = 1", Some("other desc.".to_string()))]
    #[test_case("let a = 1;", None)]
    fn test_extract_module_desc(code: &str, expected_result: Option<String>) {
        let res = extract_module_desc(code);
        assert_eq!(res, expected_result);
    }
}
