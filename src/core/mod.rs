pub mod domain;
pub mod parsing;

use colored::Colorize;
use std::process::ExitCode;

use domain::{filter_paths, FileTree};
use parsing::{list_files, Args, GitIgnore, ReadMe};

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
    let paths = list_files(&args.scripts_root);

    let paths = filter_paths(
        paths,
        &args.scripts_root,
        &args.allowed_exts,
        &args.ignore_dirs,
        &gitignore.parse_lines(),
        args.ignore_hidden,
    );

    let tree = FileTree::new().create_map(paths);
    let modified_readme = readme.update_readme(tree.render());

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
