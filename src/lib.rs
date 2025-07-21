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
    use colored::Colorize;
    use std::process::ExitCode;

    use crate::core::domain::{filter_entries, update_readme, FileTree};
    use crate::core::parsing::{list_files, Args, GitIgnore, ReadMe};

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

        let readme = ReadMe::parse(&args.readme_path)?;
        let gitignore = GitIgnore::parse(&args.gitignore_path)?;
        let entries = list_files(&args.scripts_root);

        let paths = filter_entries(
            entries,
            &args.scripts_root,
            &args.allowed_exts,
            &args.ignore_dirs,
            &gitignore.parse_lines(),
            args.ignore_hidden,
        );
        let tree = FileTree::new().create_map(paths);
        let modified_readme = update_readme(&readme, tree.render());

        if modified_readme != readme {
            if let Err(e) = modified_readme.write(&args.readme_path) {
                eprintln!("{} {}", "Failed to write README file: ".red().bold(), e);
                return Err(ExitCode::FAILURE);
            };
            println!("{}", "Modified README.md".yellow().bold());
            return Err(ExitCode::FAILURE);
        }
        println!("{}", "Nothing to modify".green().bold());
        Ok(ExitCode::SUCCESS)
    }

    mod parsing {
        use colored::Colorize;
        use regex::Regex;
        use std::{
            collections::HashSet,
            fs, io,
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

            pub fn parse_lines(&self) -> Vec<Regex> {
                self.0
                    .lines()
                    .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                    .filter_map(|pattern| {
                        let mut regex_str = String::new();

                        regex_str.push_str("(^|/)");

                        for c in pattern.chars() {
                            match c {
                                '*' => regex_str.push_str("[^/]*"),
                                '?' => regex_str.push('.'),
                                '.' => regex_str.push_str(r"\."),
                                _ => regex_str.push(c),
                            }
                        }

                        if pattern.ends_with('/') {
                            regex_str.push_str("(.*)?$");
                        } else {
                            regex_str.push('$');
                        }

                        Regex::new(&regex_str).ok()
                    })
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
        pub struct ReadMe(pub(crate) String);

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

            pub fn write(&self, path: &Path) -> Result<(), io::Error> {
                fs::write(path, &self.0)
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
        use rayon::prelude::*;
        use regex::Regex;
        use std::{
            collections::{HashMap, HashSet},
            ffi,
            path::{Path, PathBuf},
        };
        use walkdir::DirEntry;

        use crate::core::parsing::ReadMe;

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

            pub fn create_map(mut self, paths: Vec<PathBuf>) -> Self {
                for path in paths {
                    self.insert(&path);
                }
                self
            }

            pub fn render(&self) -> String {
                fn _walk(tree: &HashMap<String, FileTree>, prefix: String, out: &mut Vec<String>) {
                    let mut items: Vec<_> = tree.iter().collect();

                    items.sort_by_key(|(name, node)| (node.nodes.is_empty(), name.to_owned()));

                    for (i, (name, _)) in items.iter().enumerate() {
                        let is_last = i == items.len() - 1;
                        let connector = if is_last { "└── " } else { "├── " };
                        out.push(format!("{prefix}{connector}{name}"));

                        if let Some(subtree) = tree.get(*name) {
                            let new_prefix =
                                format!("{prefix}{}", if is_last { "    " } else { "│   " });
                            _walk(&subtree.nodes, new_prefix, out);
                        }
                    }
                }

                let mut out = Vec::new();
                _walk(&self.nodes, String::new(), &mut out);
                format!("# Repo map\n```\n{}\n::\n```", out.join("\n"))
            }
        }

        pub fn filter_entries(
            entries: Vec<DirEntry>,
            root: &PathBuf,
            allowed_exts: &HashSet<String>,
            ignore_dirs: &HashSet<String>,
            gitignored_patterns: &[Regex],
            ignore_hidden: bool,
        ) -> Vec<PathBuf> {
            #[inline(always)]
            fn _is_hidden(entry: &DirEntry) -> bool {
                entry
                    .file_name()
                    .to_str()
                    .map(|s| s.starts_with("."))
                    .unwrap_or(false)
            }

            #[inline(always)]
            fn _is_allowed_ext(entry: &DirEntry, allowed_exts: &HashSet<String>) -> bool {
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

            #[inline(always)]
            fn _is_ignored_dir(
                entry: &DirEntry,
                root: &PathBuf,
                ignore_dirs: &HashSet<String>,
            ) -> bool {
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

            #[inline(always)]
            fn _is_gitignored(path: &Path, patterns: &[Regex]) -> bool {
                let rel_str = path.to_string_lossy();
                patterns.iter().any(|re| re.is_match(&rel_str))
            }

            entries
                .into_par_iter()
                .filter(|e| !ignore_hidden || !_is_hidden(e))
                .filter(|e| _is_allowed_ext(e, allowed_exts))
                .filter(|e| !_is_ignored_dir(e, root, ignore_dirs))
                .filter_map(|e| e.path().strip_prefix(root).ok().map(|p| p.to_owned()))
                .filter(|p| !_is_gitignored(p, gitignored_patterns))
                .collect()
        }

        pub fn update_readme(readme: &ReadMe, repo_map: String) -> ReadMe {
            let pattern = Regex::new(r"(?s)(?m)^# Repo map\n```.*?^::\n```").expect("valid regex");

            let updated = if pattern.is_match(&readme.0) {
                pattern.replace(&readme.0, repo_map).into_owned()
            } else {
                format!("{}\n\n{}", readme.0, repo_map)
            };
            ReadMe(updated)
        }
    }
}
