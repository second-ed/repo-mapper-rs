pub mod adapters;
pub mod converters;
pub mod domain;
pub mod parsing;

mod test_utils;
use crate::core::adapters::FileSystem;
use crate::core::domain::{filter_dirnames, filter_paths, FileTree, RetCode};
use crate::core::parsing::{Args, GitIgnore, OutputMode, ReadMe};
use colored::Colorize;

#[allow(clippy::too_many_arguments)]
pub fn main(
    file_sys: &mut impl FileSystem,
    repo_root: String,
    readme_path: String,
    gitignore_path: String,
    allowed_exts: Vec<String>,
    ignore_dirs: Vec<String>,
    output_mode: String,
    ignore_hidden: bool,
    dirs_only: bool,
) -> Result<RetCode, RetCode> {
    let args = Args::new(
        repo_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        output_mode,
        ignore_hidden,
        dirs_only,
    );

    let gitignore = GitIgnore::parse(file_sys, &args.gitignore_path)?;
    let paths: Vec<std::path::PathBuf> = file_sys.list_files(&args.repo_root);

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

    match args.output_mode {
        OutputMode::Readme => {
            let readme = ReadMe::parse(file_sys, &args.readme_path)?;
            let modified_readme = readme.update_readme(tree.render());

            if modified_readme == readme {
                println!("{}", "Nothing to modify".green().bold());
                return Ok(RetCode::NoModification);
            }

            modified_readme
                .write(file_sys, &args.readme_path)
                .map_err(|e| {
                    eprintln!("{} {}", "Failed to write README file: ".red().bold(), e);
                    RetCode::FailedToWriteReadme
                })?;

            println!("{}", "Modified README.md".yellow().bold());
            Ok(RetCode::ModifiedReadme)
        }
        OutputMode::Shell => {
            println!("{}", tree.render());
            Ok(RetCode::NoModification)
        }
    }
}
