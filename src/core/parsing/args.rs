use crate::core::{domain::utils::to_str_type, parsing::context::Context};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Eq, PartialEq)]
pub enum OutputMode {
    Shell,
    Readme,
}

impl FromStr for OutputMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "shell" => Ok(OutputMode::Shell),
            "readme" => Ok(OutputMode::Readme),
            _ => Err(format!("Invalid output mode {}", s)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub context: Context,
    pub readme_path: PathBuf,
    pub gitignore_path: PathBuf,
    pub output_mode: OutputMode,
}

impl Args {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repo_root: String,
        readme_path: String,
        gitignore_path: String,
        allowed_exts: Vec<String>,
        ignore_dirs: Vec<String>,
        output_mode: String,
        ignore_hidden: bool,
        dirs_only: bool,
    ) -> Self {
        let readme_path: PathBuf = to_str_type(readme_path);
        let gitignore_path: PathBuf = to_str_type(gitignore_path);
        let output_mode: OutputMode = output_mode
            .parse()
            .expect("Failed to parse the output mode.");

        let context = Context::new(
            repo_root,
            allowed_exts,
            ignore_dirs,
            ignore_hidden,
            dirs_only,
        );

        Self {
            context,
            readme_path,
            gitignore_path,
            output_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, OutputMode};
    use crate::core::{domain::utils::to_collection_of_type, parsing::context::Context};
    use std::{collections::HashSet, path::PathBuf};

    #[test]
    fn test_args() {
        let inp_allowed_exts: Vec<String> = to_collection_of_type(vec!["py", "rs"]);

        let args = Args::new(
            "root".to_string(),
            "readme.md".to_string(),
            ".gitignore".to_string(),
            inp_allowed_exts,
            vec![],
            "readme".to_string(),
            true,
            false,
        );

        let allowed_exts: HashSet<String> = to_collection_of_type(vec!["py", "rs"]);
        let ignore_dirs: HashSet<String> = to_collection_of_type(Vec::<&str>::new());

        let expected_result = Args {
            context: Context {
                repo_root: PathBuf::from("root"),
                allowed_exts,
                ignore_dirs,
                ignore_hidden: true,
                dirs_only: false,
            },
            readme_path: PathBuf::from("readme.md"),
            gitignore_path: PathBuf::from(".gitignore"),
            output_mode: OutputMode::Readme,
        };

        assert_eq!(args, expected_result);
    }
}
