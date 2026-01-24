# repo-mapper-rs 🦀
[![PyPI Downloads](https://static.pepy.tech/badge/repo-mapper-rs)](https://pepy.tech/projects/repo-mapper-rs)


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
| Argument           | Type                  | Required | Default | Description                                          |
| ------------------ | --------------------- | -------- | ------- | ---------------------------------------------------- |
| `--repo-root`      | `str`                 | ✅       |  | Path to the root of the repository to scan           |
| `--readme-path`    | `str`                 | ❌       | `'./README.md'` | Path to the README file that will be modified        |
| `--gitignore-path` | `str`                 | ❌       | `'./.gitignore'` | Path to the `.gitignore` file                        |
| `--allowed-exts`   | Comma-separated `str` | ❌       | `'py,md,toml,lock,yaml,ipynb'` | Extensions to include (e.g. `'py,rs,md'`). Note this is overruled by the `.gitignore`.             |
| `--ignore-dirs`    | Comma-separated `str` | ❌       | `'.git,.venv,build,dist'` | Directories to exclude (e.g. `'.venv,target'`). If not supplied, all directories will be evaluated. Note this is overruled by the `.gitignore`.|
| `--output-mode`      | `str`        | ❌       | `'readme'` | Output mode to display the tree: `choices=['readme', 'shell']`. |
| `--ignore-hidden`  | Flag (no value)       | ❌       |  | If set, hidden files and directories will be ignored |
| `--dirs-only`      | Flag (no value)       | ❌       |  | If set, only directories and subdirectories will be mapped (useful with larger codebases). |


<!--
repo-map-desc: Installation and simple docs
-->

# New Features:
### 0.4.0
- `repo-map-desc`:
    - adding `repo-map-desc:` to a line will treat everything after `repo-map-desc:` up to the end of the line as a file description.
    - that description is added to the corresponding file in the repo map
    - it will ignore any characters before `repo-map-desc:` (e.g. comments or code)
    - only apply to the first matching line per file

# Repo map
```
├── .github
│   └── workflows
│       ├── ci.yaml
│       └── publish.yaml
├── python
│   └── repo_mapper
│       ├── __init__.py
│       └── __main__.py       # Main CLI entry point
├── src
│   ├── core
│   │   ├── domain
│   │   │   ├── file_node.rs
│   │   │   ├── file_tree.rs
│   │   │   ├── mod.rs
│   │   │   ├── repo_file.rs
│   │   │   ├── ret_codes.rs
│   │   │   ├── transform.rs  # Where the file tree is generated
│   │   │   └── utils.rs
│   │   ├── parsing
│   │   │   ├── args.rs
│   │   │   ├── context.rs
│   │   │   ├── file_text.rs
│   │   │   ├── gitignore.rs
│   │   │   ├── mod.rs
│   │   │   └── readme.rs
│   │   ├── adapters.rs
│   │   └── mod.rs
│   ├── api.rs                # The translation layer between python and rust
│   └── lib.rs
├── tests
│   └── integration_tests.rs
├── .pre-commit-config.yaml
├── Cargo.lock
├── Cargo.toml
├── README.md                 # Installation and simple docs
├── pyproject.toml
└── uv.lock

(generated with repo-mapper-rs)
::
```

# Ret codes
| RetCode               | int | description           |
| ----------------------| --- | --------------------- |
| `NoModification`      | 0   | The Repo map reflects the current state of the repo. |
| `ModifiedReadme`      | 1   | The README was updated. |
| `FailedParsingFile`   | 2   | Failed to read the file to string. |
| `FailedToWriteReadme` | 3   | Failed to write the modified README to file. |
| `InvalidFilename`     | 4   | The given `README.md` or `.gitignore` path does not match the expected basename. |
