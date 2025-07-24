use std::process::ExitCode;

use pyo3::prelude::*;

use crate::core::{adapters::RealFileSystem, main};

#[pyfunction]
fn py_main(
    repo_root: String,
    readme_path: String,
    gitignore_path: String,
    allowed_exts: Vec<String>,
    ignore_dirs: Vec<String>,
    ignore_hidden: bool,
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
    ) {
        Ok(ExitCode::SUCCESS) => Ok(0),
        Ok(ExitCode::FAILURE) => Ok(1),
        Err(ExitCode::FAILURE) => Ok(1),
        _ => Ok(-1),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn repo_mapper_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}
