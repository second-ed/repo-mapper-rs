use crate::core::{
    adapters::FileSystem,
    domain_v2::{file_node::FileNode, repo_file::RepoFile},
};
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub fn files_to_tree(
    file_sys: &mut impl FileSystem,
    paths: Vec<PathBuf>,
    allowed_exts: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    gitignored_patterns: &[Regex],
    ignore_hidden: bool,
    dirs_only: bool,
) {
    let paths = if dirs_only {
        filter_dirnames(paths.clone())
    } else {
        paths
    };

    dbg!(&paths);
    let repo_files = pathbufs_to_repo_files(paths);
    dbg!(&repo_files);

    let repo_files = filter_repo_files(
        repo_files,
        allowed_exts,
        ignore_dirs,
        gitignored_patterns,
        ignore_hidden,
    );

    dbg!(&repo_files);

    let file_nodes = repo_file_to_file_node(file_sys, repo_files);
    dbg!(file_nodes);
}

fn filter_dirnames(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter_map(|p| p.parent().map(Path::to_path_buf))
        .dedup()
        .collect()
}

fn pathbufs_to_repo_files(paths: Vec<PathBuf>) -> Vec<RepoFile> {
    paths.into_iter().map(|path| RepoFile::new(path)).collect()
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

            FileNode::from_repo_file(repo_file, desc)
        })
        .collect()
}

fn extract_module_desc(code: &str) -> Option<String> {
    code.lines().find_map(|line| {
        let (_, desc) = line.split_once("repo-map-desc:")?;
        Some(desc.trim().to_string())
    })
}
