# repo-mapper-rs 🦀
Rust implementation of [repo_mapper](https://github.com/second-ed/repo_mapper).

# What it does:
A CLI tool to scan a code repository and generate a structured file tree map, inserted into your `README.md` written in Rust.
The map is fenced inside a markdown code block under a `# Repo map` section, if one exists the existing one is replaced, else it is appended to the bottom of the `README.md`. 

Supported functionality:
- .gitignore
- file extension filtering
- directory exclusion
- ignore hidden files

# Installation
```shell
pip install repo-mapper-rs
```
Or
```shell
uv add repo-mapper-rs
```

# Example usage:
```shell
python -m repo_mapper \
  --repo-root "/path/to/my_repo" \
  --readme-path "/path/to/my_repo/README.md" \
  --gitignore-path "/path/to/my_repo/.gitignore" \
  --allowed-exts "py,rs,toml" \
  --ignore-dirs ".venv,target" \
  --ignore-hidden
```
### This command:

- Traverses `/path/to/my_repo`
- Respects files excluded in `.gitignore` or listed in `--ignore-dirs`
- Includes only files with extensions `.py`, `.rs`, `.toml`
- Skips hidden files and directories (those starting with a dot)
- Inserts or updates the `# Repo map` section in the README

# Args
| Argument           | Type                  | Required | Description                                          |
| ------------------ | --------------------- | -------- | ---------------------------------------------------- |
| `--repo-root`      | `str`                 | ✅    | Path to the root of the repository to scan           |
| `--readme-path`    | `str`                 | ✅    | Path to the README file that will be modified        |
| `--gitignore-path` | `str`                 | ✅    | Path to the `.gitignore` file                        |
| `--allowed-exts`   | Comma-separated `str` | ❌    | Extensions to include (e.g. `'py,rs,md'`). If not supplied, all extensions will be evaluated              |
| `--ignore-dirs`    | Comma-separated `str` | ❌    | Directories to exclude (e.g. `'.venv,target'`). If not supplied, all extensions will be evaluated          |
| `--ignore-hidden`  | Flag (no value)       | ❌     | If set, hidden files and directories will be ignored |


# Repo map
```
├── .github
│   └── workflows
│       ├── ci.yaml
│       └── publish.yaml
├── python
│   └── repo_mapper
│       ├── __init__.py
│       └── __main__.py
├── src
│   ├── core
│   │   ├── adapters.rs
│   │   ├── converters.rs
│   │   ├── domain.rs
│   │   ├── mod.rs
│   │   ├── parsing.rs
│   │   └── test_utils.rs
│   ├── api.rs
│   └── lib.rs
├── tests
│   └── integration_tests.rs
├── Cargo.lock
├── Cargo.toml
├── README.md
├── pyproject.toml
└── uv.lock
::
```

# Ret codes
| RetCode             | description           |
| ------------------- | --------------------- |
| `NoModification`      | The Repo Map reflects the current state of the repo. |
| `ModifiedReadme`      | The README was updated. |
| `FailedParsingFile`   | Failed to read the file to string. |
| `FailedToWriteReadme` | Failed to write the modified README to file. |
| `InvalidFilename`     | The given `README.md` or `.gitignore` path does not match the expected basename. |