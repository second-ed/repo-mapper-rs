#[allow(dead_code)]
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
