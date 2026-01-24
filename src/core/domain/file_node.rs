use crate::core::domain::repo_file::RepoFile;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct FileNode {
    pub path: PathBuf,
    pub parts: Vec<String>,
    pub desc: Option<String>,
}

impl FileNode {
    pub(crate) fn new(path: PathBuf, desc: Option<String>) -> Self {
        let parts = pathbuf_to_parts(&path);

        Self { path, parts, desc }
    }

    pub(crate) fn from_repo_file(
        repo_file: RepoFile,
        root: &PathBuf,
        desc: Option<String>,
    ) -> Self {
        FileNode::new(
            repo_file
                .path
                .as_path()
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_owned())
                .unwrap_or(repo_file.path),
            desc,
        )
    }
}

#[inline(always)]
fn pathbuf_to_parts(path: &Path) -> Vec<String> {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect()
}
