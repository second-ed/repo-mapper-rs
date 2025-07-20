use std::process::ExitCode;

use pyo3::prelude::*;

use crate::core::main;

#[pyfunction]
fn py_main(
    scripts_root: String,
    readme_path: String,
    gitignore_path: String,
    allowed_exts: Vec<String>,
    ignore_dirs: Vec<String>,
    ignore_hidden: bool,
) -> PyResult<i8> {
    match main(
        scripts_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        ignore_hidden,
    ) {
        Ok(ExitCode::SUCCESS) => Ok(0),
        Err(ExitCode::FAILURE) => Ok(1),
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
    use parsing::{list_files, Args, GitIgnore, ReadMe};
    use std::process::ExitCode;

    use crate::core::domain::FileTree;

    pub fn main(
        scripts_root: String,
        readme_path: String,
        gitignore_path: String,
        allowed_exts: Vec<String>,
        ignore_dirs: Vec<String>,
        ignore_hidden: bool,
    ) -> Result<ExitCode, ExitCode> {
        let args = Args::new(
            scripts_root,
            readme_path,
            gitignore_path,
            allowed_exts,
            ignore_dirs,
            ignore_hidden,
        );

        let gitignore = GitIgnore::parse(&args.gitignore_path)?;
        let readme = ReadMe::parse(&args.readme_path)?;

        let entries = list_files(&args.scripts_root);

        let paths = filter_dir_entries(
            entries,
            &args.scripts_root,
            &args.allowed_exts,
            &args.ignore_dirs,
            args.ignore_hidden,
        );

        let mut tree = FileTree::new();
        tree.create_map(paths);

        dbg!(tree.nodes);

        dbg!(readme);
        dbg!(gitignore);

        Ok(ExitCode::SUCCESS)
    }

    mod parsing {
        use colored::Colorize;
        use std::{
            collections::HashSet,
            fs,
            ops::Deref,
            path::{Path, PathBuf},
            process::ExitCode,
        };
        use walkdir::{DirEntry, WalkDir};

        pub struct Args {
            pub scripts_root: PathBuf,
            pub readme_path: PathBuf,
            pub gitignore_path: PathBuf,
            pub allowed_exts: HashSet<String>,
            pub ignore_dirs: HashSet<String>,
            pub ignore_hidden: bool,
        }

        impl Args {
            pub fn new(
                scripts_root: String,
                readme_path: String,
                gitignore_path: String,
                allowed_exts: Vec<String>,
                ignore_dirs: Vec<String>,
                ignore_hidden: bool,
            ) -> Self {
                let scripts_root = PathBuf::from(scripts_root);
                let readme_path = PathBuf::from(readme_path);
                let gitignore_path = PathBuf::from(gitignore_path);

                let allowed_exts: HashSet<String> = allowed_exts.into_iter().collect();
                let ignore_dirs: HashSet<String> = ignore_dirs.into_iter().collect();

                Self {
                    scripts_root,
                    readme_path,
                    gitignore_path,
                    allowed_exts,
                    ignore_dirs,
                    ignore_hidden,
                }
            }
        }

        trait FileText: Sized {
            const EXPECTED_FILENAME: &'static str;

            fn from_string(s: String) -> Self;

            fn parse(path: impl AsRef<Path>) -> Result<Self, ExitCode> {
                let path = path.as_ref().to_path_buf();
                let basename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Invalid basename");

                if basename != Self::EXPECTED_FILENAME {
                    eprintln!(
                        "{}",
                        format!(
                            "Invalid `{}` basename: `{}`",
                            Self::EXPECTED_FILENAME,
                            basename
                        )
                        .red()
                        .bold(),
                    );
                    return Err(ExitCode::FAILURE);
                }

                match fs::read_to_string(&path) {
                    Ok(contents) => Ok(Self::from_string(contents)),
                    Err(e) => {
                        eprintln!(
                            "{} {}",
                            format!("Failed to parse `{}`", Self::EXPECTED_FILENAME)
                                .red()
                                .bold(),
                            e
                        );
                        Err(ExitCode::FAILURE)
                    }
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
            pub fn parse(path: impl AsRef<Path>) -> Result<Self, ExitCode> {
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
            pub fn parse(path: impl AsRef<Path>) -> Result<Self, ExitCode> {
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
        use std::collections::HashMap;
        use std::path::Path;
        use std::{collections::HashSet, ffi, path::PathBuf};
        use walkdir::DirEntry;

        #[derive(Debug)]
        pub struct FileTree {
            pub nodes: HashMap<String, FileTree>,
        }

        impl FileTree {
            pub fn new() -> Self {
                FileTree {
                    nodes: HashMap::new(),
                }
            }

            fn insert(&mut self, path: &Path) {
                let parts = path
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string());

                let mut node = self;
                for part in parts {
                    node = node.nodes.entry(part).or_insert_with(FileTree::new);
                }
            }

            pub fn create_map(&mut self, paths: Vec<PathBuf>) {
                for path in paths {
                    self.insert(&path);
                }
            }
        }

        pub fn filter_dir_entries(
            paths: Vec<DirEntry>,
            root: &PathBuf,
            allowed_exts: &HashSet<String>,
            ignore_dirs: &HashSet<String>,
            ignore_hidden: bool,
        ) -> Vec<PathBuf> {
            paths
                .iter()
                .filter(|e| !ignore_hidden || !is_hidden(e))
                .filter(|e| is_allowed_ext(e, allowed_exts))
                .filter(|e| !is_ignored_dir(e, root, ignore_dirs))
                .filter_map(|e| {
                    e.path()
                        .strip_prefix(root.parent()?)
                        .ok()
                        .map(|p| p.to_owned())
                })
                .collect()
        }

        fn is_hidden(entry: &DirEntry) -> bool {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        }

        fn is_allowed_ext(entry: &DirEntry, allowed_exts: &HashSet<String>) -> bool {
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

        fn is_ignored_dir(entry: &DirEntry, root: &PathBuf, ignore_dirs: &HashSet<String>) -> bool {
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
