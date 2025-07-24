use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

pub trait FileSystem {
    fn list_files(&mut self, path: impl AsRef<Path>) -> Vec<PathBuf>;
    fn read_to_string(&mut self, path: &Path) -> io::Result<String>;
    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn list_files(&mut self, path: impl AsRef<Path>) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| e.path().to_owned())
            .collect()
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
    fn list_files(&mut self, path: impl AsRef<Path>) -> Vec<PathBuf> {
        self.files.keys().cloned().collect()
    }
    fn read_to_string(&mut self, path: &Path) -> io::Result<String> {
        self.operations.push(format!("read: `{}`", &path.display()));
        if let Some(contents) = self.files.get(path) {
            Ok(contents.to_owned())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }
    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error> {
        self.operations
            .push(format!("write: `{}`", &path.display()));
        self.files
            .insert(path.to_path_buf(), contents.to_string().clone());
        Ok(())
    }
}
