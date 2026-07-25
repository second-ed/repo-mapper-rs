use crate::core::domain::repo_file::RepoFile;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct FileNode {
    pub parts: Vec<String>,
    pub desc: Option<String>,
}

impl FileNode {
    pub(crate) fn new(path: &Path, desc: Option<String>) -> Self {
        let parts = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        Self { parts, desc }
    }

    pub(crate) fn from_repo_file(
        repo_file: &RepoFile,
        root: &PathBuf,
        desc: Option<String>,
    ) -> Self {
        FileNode::new(
            repo_file
                .path
                .as_path()
                .strip_prefix(root)
                .unwrap_or(&repo_file.path),
            desc,
        )
    }
}
