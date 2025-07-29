pub mod adapters;
pub mod converters;
pub mod domain;
pub mod parsing;

mod test_utils;
use crate::core::adapters::FileSystem;
use crate::core::domain::{filter_dirnames, filter_paths, FileTree, RetCode};
use crate::core::parsing::{Args, GitIgnore, ReadMe};
use colored::Colorize;

pub fn main(
    file_sys: &mut impl FileSystem,
    repo_root: String,
    readme_path: String,
    gitignore_path: String,
    allowed_exts: Vec<String>,
    ignore_dirs: Vec<String>,
    ignore_hidden: bool,
    dirs_only: bool,
) -> Result<RetCode, RetCode> {
    let args = Args::new(
        repo_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        ignore_hidden,
        dirs_only,
    );

    let readme = ReadMe::parse(file_sys, &args.readme_path)?;
    let gitignore = GitIgnore::parse(file_sys, &args.gitignore_path)?;
    let paths = file_sys.list_files(&args.repo_root);

    let paths: Vec<std::path::PathBuf> = filter_paths(
        paths,
        &args.repo_root,
        &args.allowed_exts,
        &args.ignore_dirs,
        &gitignore.parse_lines(),
        args.ignore_hidden,
    );

    let paths = if args.dirs_only {
        filter_dirnames(paths.clone())
    } else {
        paths
    };

    let tree = FileTree::new().create_map(paths);
    let modified_readme = readme.update_readme(tree.render());

    if modified_readme != readme {
        if let Err(e) = modified_readme.write(file_sys, &args.readme_path) {
            eprintln!("{} {}", "Failed to write README file: ".red().bold(), e);
            return Err(RetCode::FailedToWriteReadme);
        };
        println!("{}", "Modified README.md".yellow().bold());
        return Ok(RetCode::ModifiedReadme);
    }
    println!("{}", "Nothing to modify".green().bold());
    Ok(RetCode::NoModification)
}
