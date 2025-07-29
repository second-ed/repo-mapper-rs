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
- Includes only files with extensions `.py`, `.rs`, `.toml`. It's recommended to use this parameter to avoid unexpected files being added to the map.
- Skips hidden files and directories (those starting with a dot)
- Inserts or updates the `# Repo map` section in the README

# Args
| Argument           | Type                  | Required | Description                                          |
| ------------------ | --------------------- | -------- | ---------------------------------------------------- |
| `--repo-root`      | `str`                 | ✅    | Path to the root of the repository to scan           |
| `--readme-path`    | `str`                 | ✅    | Path to the README file that will be modified        |
| `--gitignore-path` | `str`                 | ✅    | Path to the `.gitignore` file                        |
| `--allowed-exts`   | Comma-separated `str` | ❌    | Extensions to include (e.g. `'py,rs,md'`). Note this is overruled by the `.gitignore`. Defaults to: `'py,md,toml,lock,yaml,ipynb'`.             |
| `--ignore-dirs`    | Comma-separated `str` | ❌    | Directories to exclude (e.g. `'.venv,target'`). If not supplied, all directories will be evaluated. Note this is overruled by the `.gitignore`.          |
| `--ignore-hidden`  | Flag (no value)       | ❌     | If set, hidden files and directories will be ignored |
| `--dirs-only`  | Flag (no value)       | ❌     | If set, only directories and subdirectories will be mapped (useful with larger codebases). |

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
├── .pre-commit-config.yaml
├── Cargo.lock
├── Cargo.toml
├── README.md
├── pyproject.toml
└── uv.lock
::
```

# Ret codes
| RetCode               | int | description           |
| ----------------------| --- | --------------------- |
| `NoModification`      | 0   | The Repo Map reflects the current state of the repo. |
| `ModifiedReadme`      | 1   | The README was updated. |
| `FailedParsingFile`   | 2   | Failed to read the file to string. |
| `FailedToWriteReadme` | 3   | Failed to write the modified README to file. |
| `InvalidFilename`     | 4   | The given `README.md` or `.gitignore` path does not match the expected basename. |
