use std::{collections::HashSet, path::PathBuf};

use regex::Regex;

pub fn get_mock_repo_vec() -> Vec<&'static str> {
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

pub fn create_pathbuf_vec(inp_vec: Vec<&str>) -> Vec<PathBuf> {
    inp_vec.iter().map(PathBuf::from).collect()
}

pub fn create_hashset(inp_vec: Vec<&str>) -> HashSet<String> {
    inp_vec
        .into_iter()
        .map(str::to_string)
        .collect::<HashSet<_>>()
}

pub fn create_gitignore_patterns(inp_vec: Vec<&str>) -> Vec<Regex> {
    inp_vec
        .into_iter()
        .filter_map(|s| Regex::new(s).ok())
        .collect::<Vec<Regex>>()
}
