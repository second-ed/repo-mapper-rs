use std::process::ExitCode;

use pyo3::prelude::*;

use crate::core::main;

#[pyfunction]
fn py_main(scripts_root: String, readme_path: String, gitignore_path: String) -> PyResult<i8> {
    match main(scripts_root, readme_path, gitignore_path) {
        ExitCode::SUCCESS => Ok(0),
        ExitCode::FAILURE => Ok(1),
        _ => Ok(-1),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn repo_mapper_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}

mod core {
    use domain::filter_dir_entries;
    use parsing::{list_files, GitIgnore, ReadMe};
    use std::{collections::HashSet, path::PathBuf, process::ExitCode};

    use colored::Colorize;

    pub fn main(scripts_root: String, readme_path: String, gitignore: String) -> ExitCode {
        let scripts_root = PathBuf::from(scripts_root);
        let print_fail_and_exit = |msg: String, e: std::io::Error| -> ExitCode {
            eprintln!("{} {}", msg.red().bold(), e);
            ExitCode::FAILURE
        };

        let gitignore = match GitIgnore::parse(gitignore) {
            Ok(contents) => contents,
            Err((msg, e)) => return print_fail_and_exit(msg, e),
        };

        let readme = match ReadMe::parse(readme_path) {
            Ok(contents) => contents,
            Err((msg, e)) => return print_fail_and_exit(msg, e),
        };

        let entries = list_files(&scripts_root);

        let allowed_exts: HashSet<&str> = ["py", "rs", "md"].iter().cloned().collect();
        let ignore_dirs: HashSet<&str> = [".venv"].iter().cloned().collect();

        dbg!(filter_dir_entries(
            entries,
            &scripts_root,
            &allowed_exts,
            &ignore_dirs,
            true
        ));

        ExitCode::SUCCESS
    }

    mod parsing {
        use std::{fs, io, ops::Deref, path::Path};
        use walkdir::{DirEntry, WalkDir};

        trait FileText: Sized {
            const EXPECTED_FILENAME: &'static str;

            fn from_string(s: String) -> Self;

            fn parse(path: impl AsRef<Path>) -> Result<Self, (String, io::Error)> {
                let path = path.as_ref().to_path_buf();
                let basename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Invalid basename");

                if basename != Self::EXPECTED_FILENAME {
                    return Err((
                        format!(
                            "Invalid `{}` basename: `{}`",
                            Self::EXPECTED_FILENAME,
                            basename
                        ),
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Expected `{}`", Self::EXPECTED_FILENAME),
                        ),
                    ));
                }

                match fs::read_to_string(&path) {
                    Ok(contents) => Ok(Self::from_string(contents)),
                    Err(e) => Err((format!("Failed to parse `{}`", Self::EXPECTED_FILENAME), e)),
                }
            }
        }

        #[derive(Debug, Eq, PartialEq)]
        pub struct GitIgnore(String);

        impl FileText for GitIgnore {
            const EXPECTED_FILENAME: &'static str = ".gitignore";

            fn from_string(s: String) -> Self {
                GitIgnore(s)
            }
        }

        impl GitIgnore {
            pub fn parse(path: impl AsRef<Path>) -> Result<Self, (String, io::Error)> {
                <Self as FileText>::parse(path)
            }

            pub fn process_lines(&self) -> Vec<String> {
                self.0
                    .lines()
                    .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                    .map(str::to_string)
                    .collect()
            }
        }

        impl Deref for GitIgnore {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[derive(Debug, Eq, PartialEq)]
        pub struct ReadMe(String);

        impl FileText for ReadMe {
            const EXPECTED_FILENAME: &'static str = "README.md";

            fn from_string(s: String) -> Self {
                ReadMe(s)
            }
        }

        impl ReadMe {
            pub fn parse(path: impl AsRef<Path>) -> Result<Self, (String, io::Error)> {
                <Self as FileText>::parse(path)
            }
        }

        impl Deref for ReadMe {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        pub fn list_files(path: impl AsRef<Path>) -> Vec<DirEntry> {
            WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .collect()
        }
    }

    mod domain {
        use std::{
            collections::HashSet,
            ffi,
            path::{Path, PathBuf},
        };
        use walkdir::DirEntry;

        pub fn filter_dir_entries(
            paths: Vec<DirEntry>,
            root: &PathBuf,
            allowed_exts: &HashSet<&str>,
            ignore_dirs: &HashSet<&str>,
            ignore_hidden: bool,
        ) -> Vec<DirEntry> {
            paths
                .iter()
                .filter(|e| !ignore_hidden || !is_hidden(e))
                .filter(|e| is_allowed_ext(e, allowed_exts))
                .filter(|e| !is_ignored_dir(e, root, ignore_dirs))
                .map(|e| e.to_owned())
                .collect()
        }

        fn is_hidden(entry: &DirEntry) -> bool {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        }

        fn is_allowed_ext(entry: &DirEntry, allowed_exts: &HashSet<&str>) -> bool {
            if allowed_exts.is_empty() {
                return true;
            }
            entry
                .path()
                .extension()
                .and_then(ffi::OsStr::to_str)
                .map(|ext| allowed_exts.contains(ext))
                .unwrap_or(false)
        }

        fn is_ignored_dir(entry: &DirEntry, root: &PathBuf, ignore_dirs: &HashSet<&str>) -> bool {
            if ignore_dirs.is_empty() {
                return true;
            }
            match entry.path().strip_prefix(root) {
                Ok(stripped) => stripped.ancestors().any(|anc| {
                    anc.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| ignore_dirs.contains(name))
                        .unwrap_or(false)
                }),
                Err(_) => true,
            }
        }
    }
}
