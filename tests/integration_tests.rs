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

struct InputData {
    readme_path: String,
    gitignore_path: String,
    allowed_exts: Vec<String>,
    ignore_dirs: Vec<String>,
    ignore_hidden: bool,
    dirs_only: bool,
}

impl InputData {
    fn new(
        readme_path: &str,
        gitignore_path: &str,
        allowed_exts: Vec<&str>,
        ignore_dirs: Vec<&str>,
        ignore_hidden: bool,
        dirs_only: bool,
    ) -> Self {
        let readme_path = readme_path.to_string();
        let gitignore_path = gitignore_path.to_string();
        let allowed_exts: Vec<String> = to_collection_of_type(allowed_exts);
        let ignore_dirs: Vec<String> = to_collection_of_type(ignore_dirs);

        Self {
            readme_path,
            gitignore_path,
            allowed_exts,
            ignore_dirs,
            ignore_hidden,
            dirs_only,
        }
    }
}

struct ExpectedResult {
    ret_code: Result<RetCode, RetCode>,
    readme: String,
}

impl ExpectedResult {
    fn new(ret_code: Result<RetCode, RetCode>, readme: &str) -> Self {
        Self {
            ret_code,
            readme: readme.to_string(),
        }
    }
}

#[test_case(
    InputData::new("fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml"],
    vec![".venv", "target"],
    true, false),
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n\n(generated with repo-mapper-rs)\n::\n```",
    ExpectedResult::new(Ok(RetCode::NoModification),
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n\n(generated with repo-mapper-rs)\n::\n```") ;
    "Ensure returns Ok(RetCode::NoModification)) when README is not modified"
)]
#[test_case(
    InputData::new("fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml"],
    vec![".venv", "target"],
    true, true),
    "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n::\n```",
    ExpectedResult::new(Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n└── src\n\n(generated with repo-mapper-rs)\n::\n```" );
    "Ensure only shows directories if dirs_only is true"
)]
#[test_case(
    InputData::new("fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec!["rs", "md", "toml", "py"],
    vec![],
    true, false),
    "# Some readme\n`\n\n# Repo map\n```\n├── .venv\n│   └── site-packages\n│       └── some_package.py\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```",
    ExpectedResult::new(Ok(RetCode::ModifiedReadme),
    "# Some readme\n`\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```") ;
    "Ensure doesn't ignore directories if given empty vec"
)]
#[test_case(
    InputData::new("fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![".venv", "src"],
    true, false),
    "# Some readme\n",
    ExpectedResult::new(Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```") ;
    "Ensure return Ok(RetCode::ModifiedReadme) if it modifies the README"
)]
#[test_case(
    InputData::new("fake/repo/root/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![".venv", "src"],
    false, false),
    "# Some readme\n",
    ExpectedResult::new(Ok(RetCode::ModifiedReadme),
    "# Some readme\n\n\n# Repo map\n```\n├── secrets\n│   └── .env\n├── .gitignore\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```") ;
    "Ensure does not skip hidden file"
)]
#[test_case(
    InputData::new("fake/repo/root/WRONG_README.txt",
    "fake/repo/root/.gitignore",
    vec![],
    vec![],
    true, false),
    "# Some readme\n",
    ExpectedResult::new(Err(RetCode::InvalidFilename),
    "# Some readme\n" );
    "Ensure Err(RetCode::FailedParsingFile) if not pointed to a valid README"
)]
#[test_case(
    InputData::new("fake/repo/root/docs/README.md",
    "fake/repo/root/.gitignore",
    vec![],
    vec![],
    true, false),
    "# Some readme\n",
    ExpectedResult::new(Err(RetCode::FailedParsingFile),
    "# Some readme\n") ;
    "Ensure Err(FAILURE) if not pointed to an invalid file"
)]
#[test_case(
    InputData::new("fake/repo/root/README.md",
    "fake/repo/root/.gitdonotignore",
    vec![],
    vec![],
    true, false),
    "# Some readme\n",
    ExpectedResult::new(Err(RetCode::InvalidFilename),
    "# Some readme\n" );
    "Ensure Err(FAILURE) if not pointed to valid gitignore."
)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn test_modify_readme(
    input_data: InputData,
    current_readme: &str,
    expected_result: ExpectedResult,
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

    let exit_code = main(
        &mut file_sys,
        repo_root,
        input_data.readme_path,
        input_data.gitignore_path,
        input_data.allowed_exts,
        input_data.ignore_dirs,
        "readme",
        input_data.ignore_hidden,
        input_data.dirs_only,
    );

    let readme_pathbuf = PathBuf::from("fake/repo/root/README.md");

    assert_eq!(exit_code, expected_result.ret_code);
    assert_eq!(
        file_sys.files.get(&readme_pathbuf).unwrap().to_owned(),
        expected_result.readme
    );
}
