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
    current_readme: String,
}

impl InputData {
    fn new(
        readme_path: &str,
        gitignore_path: &str,
        allowed_exts: Vec<&str>,
        ignore_dirs: Vec<&str>,
        ignore_hidden: bool,
        dirs_only: bool,
        current_readme: &[&str],
    ) -> Self {
        let readme_path = format!("{ROOT}/{readme_path}");
        let gitignore_path = format!("{ROOT}/{gitignore_path}");
        let allowed_exts: Vec<String> = to_collection_of_type(allowed_exts);
        let ignore_dirs: Vec<String> = to_collection_of_type(ignore_dirs);
        let current_readme = current_readme.join("\n");

        Self {
            readme_path,
            gitignore_path,
            allowed_exts,
            ignore_dirs,
            ignore_hidden,
            dirs_only,
            current_readme,
        }
    }
}

struct ExpectedResult {
    ret_code: Result<RetCode, RetCode>,
    readme: String,
}

impl ExpectedResult {
    fn new(ret_code: Result<RetCode, RetCode>, readme: &[&str]) -> Self {
        Self {
            ret_code,
            readme: readme.join("\n"),
        }
    }
}

fn given_valid_input_data_when_called_then_should_not_modify_the_readme(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n\n(generated with repo-mapper-rs)\n::\n```"];
    let expected_readme = ["# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n\n(generated with repo-mapper-rs)\n::\n```"];

    (
        InputData::new(
            "README.md",
            ".gitignore",
            vec!["rs", "md", "toml"],
            vec![".venv", "target"],
            true,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Ok(RetCode::NoModification), &expected_readme),
    )
}

fn given_valid_input_data_when_called_with_dirs_only_then_should_not_modify_the_readme(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n::\n```"];
    let expected_readme = [
        "# Some readme\n\n\n# Repo map\n```\n└── src\n\n(generated with repo-mapper-rs)\n::\n```",
    ];

    (
        InputData::new(
            "README.md",
            ".gitignore",
            vec!["rs", "md", "toml"],
            vec![".venv", "target"],
            true,
            true,
            &current_readme,
        ),
        ExpectedResult::new(Ok(RetCode::ModifiedReadme), &expected_readme),
    )
}

fn given_empty_ignore_dirs_when_called_then_should_not_ignore_directories(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n`\n\n# Repo map\n```\n├── .venv\n│   └── site-packages\n│       └── some_package.py\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```"];
    let expected_readme = ["# Some readme\n`\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```"];

    (
        InputData::new(
            "README.md",
            ".gitignore",
            vec!["rs", "md", "toml", "py"],
            vec![],
            true,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Ok(RetCode::ModifiedReadme), &expected_readme),
    )
}

fn given_ignored_directories_when_called_then_should_modify_the_readme(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n"];
    let expected_readme = ["# Some readme\n\n\n# Repo map\n```\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```",];

    (
        InputData::new(
            "README.md",
            ".gitignore",
            vec![],
            vec![".venv", "src"],
            true,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Ok(RetCode::ModifiedReadme), &expected_readme),
    )
}

fn given_hidden_files_when_called_without_ignore_hidden_then_should_include_them(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n"];
    let expected_readme = ["# Some readme\n\n\n# Repo map\n```\n├── secrets\n│   └── .env\n├── .gitignore\n├── Cargo.toml\n├── README.md\n└── scratch.py\n\n(generated with repo-mapper-rs)\n::\n```"];

    (
        InputData::new(
            "README.md",
            ".gitignore",
            vec![],
            vec![".venv", "src"],
            false,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Ok(RetCode::ModifiedReadme), &expected_readme),
    )
}

fn given_invalid_readme_filename_when_called_then_should_return_invalid_filename(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n"];
    let expected_readme = ["# Some readme\n"];

    (
        InputData::new(
            "WRONG_README.txt",
            ".gitignore",
            vec![],
            vec![],
            true,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Err(RetCode::InvalidFilename), &expected_readme),
    )
}

fn given_missing_readme_when_called_then_should_return_failed_parsing_file(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n"];
    let expected_readme = ["# Some readme\n"];

    (
        InputData::new(
            "docs/README.md",
            ".gitignore",
            vec![],
            vec![],
            true,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Err(RetCode::FailedParsingFile), &expected_readme),
    )
}

fn given_invalid_gitignore_filename_when_called_then_should_return_invalid_filename(
) -> (InputData, ExpectedResult) {
    let current_readme = ["# Some readme\n"];
    let expected_readme = ["# Some readme\n"];

    (
        InputData::new(
            "README.md",
            ".gitdonotignore",
            vec![],
            vec![],
            true,
            false,
            &current_readme,
        ),
        ExpectedResult::new(Err(RetCode::InvalidFilename), &expected_readme),
    )
}

#[test_case(
    given_valid_input_data_when_called_then_should_not_modify_the_readme() ;
    "Ensure returns Ok(RetCode::NoModification)) when README is not modified"
)]
#[test_case(
    given_valid_input_data_when_called_with_dirs_only_then_should_not_modify_the_readme() ;
    "Ensure only shows directories if dirs_only is true"
)]
#[test_case(
    given_empty_ignore_dirs_when_called_then_should_not_ignore_directories() ;
    "Ensure doesn't ignore directories if given empty vec"
)]
#[test_case(
    given_ignored_directories_when_called_then_should_modify_the_readme() ;
    "Ensure return Ok(RetCode::ModifiedReadme) if it modifies the README"
)]
#[test_case(
    given_hidden_files_when_called_without_ignore_hidden_then_should_include_them() ;
    "Ensure does not skip hidden file"
)]
#[test_case(
    given_invalid_readme_filename_when_called_then_should_return_invalid_filename() ;
    "Ensure Err(RetCode::FailedParsingFile) if not pointed to a valid README"
)]
#[test_case(
    given_missing_readme_when_called_then_should_return_failed_parsing_file() ;
    "Ensure Err(FAILURE) if not pointed to an invalid file"
)]
#[test_case(
    given_invalid_gitignore_filename_when_called_then_should_return_invalid_filename() ;
    "Ensure Err(FAILURE) if not pointed to valid gitignore."
)]
#[allow(clippy::needless_pass_by_value)]
fn test_modify_readme(test_data: (InputData, ExpectedResult)) {
    let (input_data, expected_result) = test_data;

    let files = [
        ("src/main.rs", "let x = 1;"),
        ("src/lib.rs", "use std;"),
        ("Cargo.toml", ""),
        ("README.md", &input_data.current_readme),
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

    let readme_pathbuf = Path::new(ROOT).join("README.md");

    assert_eq!(exit_code, expected_result.ret_code);
    assert_eq!(
        file_sys.files.get(&readme_pathbuf).unwrap().to_owned(),
        expected_result.readme
    );
}
