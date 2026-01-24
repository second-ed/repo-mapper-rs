pub mod adapters;
pub mod domain;
pub mod parsing;
use crate::core::{
    adapters::FileSystem,
    domain::{ret_codes::RetCode, transform::pathbufs_to_filetree},
    parsing::{
        args::{Args, OutputMode},
        gitignore::GitIgnore,
        readme::ReadMe,
    },
};
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

    let gitignored_patterns = GitIgnore::parse(file_sys, &args.gitignore_path)?.parse_lines();
    let paths: Vec<std::path::PathBuf> = file_sys.list_files(&args.context.repo_root);

    let tree = pathbufs_to_filetree(
        file_sys,
        paths,
        &args.context.repo_root,
        &args.context.allowed_exts,
        &args.context.ignore_dirs,
        &gitignored_patterns,
        args.context.ignore_hidden,
        args.context.dirs_only,
    );

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
