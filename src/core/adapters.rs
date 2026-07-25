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
            .filter_entry(|e| continue_walking(e.path(), ignore_dirs, ignore_hidden))
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
        self.files
            .keys()
            .filter(|p| p.starts_with(&path) && continue_walking(p, ignore_dirs, ignore_hidden))
            .cloned()
            .collect()
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

    if is_hidden(ignore_hidden, name) || is_ignored_dir(ignore_dirs, name) {
        return false;
    }
    true
}

#[inline]
fn is_hidden(ignore_hidden: bool, name: Option<&str>) -> bool {
    ignore_hidden && name.is_some_and(|name| name.starts_with('.'))
}

#[inline]
fn is_ignored_dir(ignore_dirs: &HashSet<String>, name: Option<&str>) -> bool {
    !ignore_dirs.is_empty() && name.is_some_and(|name| ignore_dirs.contains(name))
}
