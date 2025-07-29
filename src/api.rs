use crate::core::{adapters::RealFileSystem, domain::RetCode, main};
use pyo3::prelude::*;

#[pyfunction]
fn py_main(
    repo_root: String,
    readme_path: String,
    gitignore_path: String,
    allowed_exts: Vec<String>,
    ignore_dirs: Vec<String>,
    ignore_hidden: bool,
    dirs_only: bool,
) -> PyResult<i8> {
    let mut file_sys = RealFileSystem;

    match main(
        &mut file_sys,
        repo_root,
        readme_path,
        gitignore_path,
        allowed_exts,
        ignore_dirs,
        ignore_hidden,
        dirs_only,
    ) {
        Ok(RetCode::NoModification) => Ok(0),
        Ok(RetCode::ModifiedReadme) => Ok(1),
        Err(RetCode::FailedParsingFile) => Ok(2),
        Err(RetCode::FailedToWriteReadme) => Ok(3),
        Err(RetCode::InvalidFilename) => Ok(4),
        _ => Ok(-1),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn repo_mapper_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}
