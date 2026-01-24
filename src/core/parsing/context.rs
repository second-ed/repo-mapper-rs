use std::{collections::HashSet, path::PathBuf};

use crate::core::domain::utils::{to_collection_of_type, to_str_type};

#[derive(Debug, Eq, PartialEq)]
pub struct Context {
    pub repo_root: PathBuf,
    pub allowed_exts: HashSet<String>,
    pub ignore_dirs: HashSet<String>,
    pub ignore_hidden: bool,
    pub dirs_only: bool,
}

impl Context {
    pub fn new(
        repo_root: String,
        allowed_exts: Vec<String>,
        ignore_dirs: Vec<String>,
        ignore_hidden: bool,
        dirs_only: bool,
    ) -> Self {
        let repo_root: PathBuf = to_str_type(repo_root);
        let allowed_exts: HashSet<String> = to_collection_of_type(allowed_exts);
        let ignore_dirs: HashSet<String> = to_collection_of_type(ignore_dirs);

        Self {
            repo_root,
            allowed_exts,
            ignore_dirs,
            ignore_hidden,
            dirs_only,
        }
    }
}
