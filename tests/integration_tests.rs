use std::{collections::HashMap, path::PathBuf};

use repo_mapper_rs::core::{
    adapters::FakeFileSystem, converters::to_strings, domain::RetCode, main,
};
use test_case::test_case;

#[test_case(
    "fake/repo/root/README.md", "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml"],
    vec![".venv", "target"],
    true,
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n::\n```",
    Ok(RetCode::NoModification),
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n::\n```" ;
    "Ensure returns Ok(RetCode::NoModification)) when README is not modified"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml", "py"],
    vec![],
    true,
    "# Some readme\n`\n\n# Repo map\n```\n├── .venv\n│   └── site-packages\n│       └── some_package.py\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n::\n```",
    Ok(RetCode::NoModification),
    "# Some readme\n`\n\n# Repo map\n```\n├── .venv\n│   └── site-packages\n│       └── some_package.py\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n::\n```" ;
    "Ensure doesn't ignore directories if given empty vec"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![".venv", "src"],
    true,
    "# Some readme\n",
    Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n├── Cargo.toml\n├── README.md\n└── scratch.py\n::\n```" ;
    "Ensure return Ok(RetCode::ModifiedReadme) if it modifies the README"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![".venv", "src"],
    false,
    "# Some readme\n",
    Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n├── secrets\n│   └── .env\n├── .gitignore\n├── Cargo.toml\n├── README.md\n└── scratch.py\n::\n```" ;
    "Ensure does not skip hidden file"
)]
#[test_case(
    "fake/repo/root/WRONG_README.txt",
    "fake/repo/root/.gitignore",
    vec![],
    vec![],
    true,
    "# Some readme\n",
    Err(RetCode::InvalidFilename),
    "# Some readme\n" ;
    "Ensure Err(RetCode::FailedParsingFile) if not pointed to a valid README"
)]
#[test_case(
    "fake/repo/root/docs/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![],
    true,
    "# Some readme\n",
    Err(RetCode::FailedParsingFile),
    "# Some readme\n" ;
    "Ensure Err(FAILURE) if not pointed to an invalid file"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitdonotignore",
    vec![],
    vec![],
    true,
    "# Some readme\n",
    Err(RetCode::InvalidFilename),
    "# Some readme\n" ;
    "Ensure Err(FAILURE) if not pointed to valid gitignore."
)]
#[allow(clippy::too_many_arguments)]
fn test_modify_readme(
    readme_path: &str,
    gitignore_path: &str,
    allowed_exts: Vec<&str>,
    ignore_dirs: Vec<&str>,
    ignore_hidden: bool,
    current_readme: &str,
    expected_result: Result<RetCode, RetCode>,
    expected_readme: &str,
) {
    let files = vec![
        ("fake/repo/root/src/main.rs", "let x = 1;"),
        ("fake/repo/root/src/lib.rs", "use std;"),
        ("fake/repo/root/Cargo.toml", ""),
        ("fake/repo/root/README.md", current_readme),
        ("fake/repo/root/.gitignore", "target/"),
        ("fake/repo/root/target/some_build.rs", ""),
        ("fake/repo/root/.venv/site-packages/some_package.py", ""),
        ("fake/repo/root/scratch.py", ""),
        ("fake/repo/root/secrets/.env", ""),
    ]
    .into_iter()
    .map(|(k, v)| (PathBuf::from(k), v.to_string()))
    .collect::<HashMap<PathBuf, String>>();

    let mut file_sys = FakeFileSystem::new(files);

    let repo_root = "fake/repo/root".to_string();
    let readme_path = readme_path.to_string();
    let gitignore_path = gitignore_path.to_string();
    let allowed_exts = to_strings(allowed_exts);
    let ignore_dirs = to_strings(ignore_dirs);

    let exit_code = main(
        &mut file_sys,
        repo_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        ignore_hidden,
    );

    let readme_pathbuf = PathBuf::from("fake/repo/root/README.md");

    assert_eq!(exit_code, expected_result);
    assert_eq!(
        file_sys.files.get(&readme_pathbuf).unwrap().to_owned(),
        expected_readme.to_string()
    );
}
