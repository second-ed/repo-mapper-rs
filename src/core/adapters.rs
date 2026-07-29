use crate::core::domain::repo_entry::is_hidden;
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub trait FileSystem {
    fn list_files(
        &mut self,
        path: impl AsRef<Path>,
        ignore_dirs: &HashSet<String>,
        ignore_hidden: bool,
    ) -> Vec<PathBuf>;

    fn is_file(&self, path: &Path) -> bool;

    fn read_to_string(&mut self, path: &Path) -> io::Result<String>;

    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn list_files(
        &mut self,
        path: impl AsRef<Path>,
        ignore_dirs: &HashSet<String>,
        ignore_hidden: bool,
    ) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                !e.file_type().is_dir() || continue_walking(e.path(), ignore_dirs, ignore_hidden)
            })
            .filter_map(Result::ok)
            .map(|e| e.path().to_owned())
            .collect()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read_to_string(&mut self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error> {
        fs::write(path, contents)
    }
}

pub struct FakeFileSystem {
    pub files: HashMap<PathBuf, String>,
    pub operations: Vec<String>,
}

impl FakeFileSystem {
    #[must_use]
    pub fn new(files: HashMap<PathBuf, String>) -> Self {
        Self {
            files,
            operations: Vec::new(),
        }
    }
}

impl Default for FakeFileSystem {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl FileSystem for FakeFileSystem {
    fn list_files(
        &mut self,
        path: impl AsRef<Path>,
        ignore_dirs: &HashSet<String>,
        ignore_hidden: bool,
    ) -> Vec<PathBuf> {
        let mut files = vec![path.as_ref().to_path_buf()];
        files.extend(
            self.files
                .keys()
                .filter(|p| {
                    p.starts_with(&path)
                        && p.parent()
                            .into_iter()
                            .flat_map(Path::ancestors)
                            .all(|parent| continue_walking(parent, ignore_dirs, ignore_hidden))
                })
                .cloned()
                .collect::<Vec<PathBuf>>(),
        );
        files
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_to_string(&mut self, path: &Path) -> io::Result<String> {
        self.operations.push(format!("read: `{}`", path.display()));
        if let Some(contents) = self.files.get(path) {
            Ok(contents.to_owned())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error> {
        self.operations.push(format!("write: `{}`", path.display()));
        self.files
            .insert(path.to_path_buf(), contents.to_string().clone());
        Ok(())
    }
}

fn continue_walking(path: &Path, ignore_dirs: &HashSet<String>, ignore_hidden: bool) -> bool {
    let name = path.file_name().and_then(|name| name.to_str());
    let ignored_dir = name.is_some_and(|name| ignore_dirs.contains(name));

    !(is_hidden(path, ignore_hidden) || ignored_dir)
}

#[cfg(test)]
mod tests {
    use crate::core::{adapters::continue_walking, domain::utils::to_collection_of_type};
    use std::{collections::HashSet, path::Path};
    use test_case::test_case;

    #[test_case("walking/the/tree/to/ignore", vec!["ignore"], false, false ; "given the path as walked to an ignore dir, when continue_walking is called, then returns false")]
    #[test_case("walking/the/tree/to/ignore", vec!["ignore"], true, false ; "given the path as walked to an ignore dir, when continue_walking is called with ignore_hidden, then returns false")]
    #[test_case("walking/the/tree/to/.hidden_dir", vec!["ignore"], true, false ; "given a hidden path, when ignore_hidden is true, then return false")]
    #[test_case("walking/the/tree/to/.hidden_dir", vec!["ignore"], false, true ; "given a hidden path, when ignore_hidden is false, then return true")]
    fn test_continue_walking(
        inp_path: &str,
        inp_ignore_dirs: Vec<&str>,
        ignore_hidden: bool,
        expected_result: bool,
    ) {
        let ignore_dirs: HashSet<String> = to_collection_of_type(inp_ignore_dirs);
        let res = continue_walking(Path::new(inp_path), &ignore_dirs, ignore_hidden);
        assert_eq!(res, expected_result);
    }
}
