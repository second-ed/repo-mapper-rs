use repo_mapper_rs::core::{
    adapters::FakeFileSystem,
    domain::{ret_codes::RetCode, utils::to_collection_of_type},
    main,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use test_case::test_case;

const ROOT: &str = "fake/repo/root";

fn populate_file_sys(files: &[(&str, &str)]) -> FakeFileSystem {
    let file_map = files
        .iter()
        .map(|(k, v)| (Path::new(ROOT).join(k), v.to_string()))
        .collect::<HashMap<PathBuf, String>>();
    FakeFileSystem::new(file_map)
}

#[test_case(
    "fake/repo/root/README.md", "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml"],
    vec![".venv", "target"],
    true, false,
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n\n(generated with repo-mapper-rs)\n::\n```",
    Ok(RetCode::NoModification),
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n\n(generated with repo-mapper-rs)\n::\n```" ;
    "Ensure returns Ok(RetCode::NoModification)) when README is not modified"
)]
#[test_case(
    "fake/repo/root/README.md", "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml"],
    vec![".venv", "target"],
    true, true,
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n::\n```",
    Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n└── src\n\n(generated with repo-mapper-rs)\n::\n```" ;
    "Ensure only shows directories if dirs_only is true"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml", "py"],
    vec![],
    true, false,
    "# Some readme\n`\n\n# Repo map\n```\n├── .venv\n│   └── site-packages\n│       └── some_package.py\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```",
    Ok(RetCode::ModifiedReadme),
    "# Some readme\n`\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```" ;
    "Ensure doesn't ignore directories if given empty vec"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![".venv", "src"],
    true, false,
    "# Some readme\n",
    Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```" ;
    "Ensure return Ok(RetCode::ModifiedReadme) if it modifies the README"
)]
#[test_case(
    "fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![".venv", "src"],
    false, false,
    "# Some readme\n",
    Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n├── secrets\n│   └── .env\n├── .gitignore\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```" ;
    "Ensure does not skip hidden file"
)]
#[test_case(
    "fake/repo/root/WRONG_README.txt",
    "fake/repo/root/.gitignore",
    vec![],
    vec![],
    true, false,
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
    true, false,
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
    true, false,
    "# Some readme\n",
    Err(RetCode::InvalidFilename),
    "# Some readme\n" ;
    "Ensure Err(FAILURE) if not pointed to valid gitignore."
)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn test_modify_readme(
    readme_path: &str,
    gitignore_path: &str,
    allowed_exts: Vec<&str>,
    ignore_dirs: Vec<&str>,
    ignore_hidden: bool,
    dirs_only: bool,
    current_readme: &str,
    expected_result: Result<RetCode, RetCode>,
    expected_readme: &str,
) {
    let files = [
        ("src/main.rs", "let x = 1;"),
        ("src/lib.rs", "use std;"),
        ("Cargo.toml", ""),
        ("README.md", current_readme),
        (".gitignore", "target/"),
        ("target/some_build.rs", ""),
        (".venv/site-packages/some_package.py", ""),
        ("scratch.py", ""),
        ("secrets/.env", ""),
    ];

    let mut file_sys = populate_file_sys(&files);

    let repo_root = ROOT.to_string();
    let readme_path = readme_path.to_string();
    let gitignore_path = gitignore_path.to_string();
    let allowed_exts: Vec<String> = to_collection_of_type(allowed_exts);
    let ignore_dirs: Vec<String> = to_collection_of_type(ignore_dirs);

    let exit_code = main(
        &mut file_sys,
        repo_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        "readme",
        ignore_hidden,
        dirs_only,
    );

    let readme_pathbuf = PathBuf::from("fake/repo/root/README.md");

    assert_eq!(exit_code, expected_result);
    assert_eq!(
        file_sys.files.get(&readme_pathbuf).unwrap().to_owned(),
        expected_readme.to_string()
    );
}
