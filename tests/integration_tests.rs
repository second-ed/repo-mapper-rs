use std::{collections::HashMap, path::PathBuf, process::ExitCode};

use repo_mapper_rs::core::{adapters::FakeFileSystem, converters::to_strings, main};

#[test]
fn test_modify_readme() {
    let files = vec![
        ("fake/repo/root/src/main.rs", "let x = 1;"),
        ("fake/repo/root/src/lib.rs", "use std;"),
        ("fake/repo/root/Cargo.toml", ""),
        ("fake/repo/root/README.md", "# Some readme\n"),
        ("fake/repo/root/.gitignore", "target/"),
        ("fake/repo/root/target/some_build.rs", ""),
        ("fake/repo/root/scratch.py", ""),
    ]
    .into_iter()
    .map(|(k, v)| (PathBuf::from(k), v.to_string()))
    .collect::<HashMap<PathBuf, String>>();

    let mut file_sys = FakeFileSystem::new(files);

    let repo_root = "fake/repo/root".to_string();
    let readme_path = "fake/repo/root/README.md".to_string();
    let gitignore_path = "fake/repo/root/.gitignore".to_string();
    let allowed_exts = to_strings(["py", "rs"]);
    let ignore_dirs = to_strings([".venv", "target"]);
    let ignore_hidden = false;

    let exit_code = main(
        &mut file_sys,
        repo_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        ignore_hidden,
    );
    dbg!(&file_sys
        .files
        .get(&PathBuf::from("fake/repo/root/README.md")));

    println!(
        "{}",
        &file_sys
            .files
            .get(&PathBuf::from("fake/repo/root/README.md"))
            .unwrap()
    );
    // FAILURE because it modifies the readme
    assert_eq!(exit_code, Ok(ExitCode::FAILURE));
}
