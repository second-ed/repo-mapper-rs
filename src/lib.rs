use std::{path::Path, process::ExitCode};

use pyo3::prelude::*;

use crate::core::main;

#[pyfunction]
fn py_main(scripts_root: String, readme_path: String) -> PyResult<i8> {
    match main(scripts_root, Path::new(&readme_path)) {
        ExitCode::SUCCESS => Ok(0),
        ExitCode::FAILURE => Ok(1),
        _ => Ok(-1),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn repo_mapper_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}

mod core {
    use std::{path::Path, process::ExitCode};

    pub fn main(scripts_root: String, readme_path: &Path) -> ExitCode {
        println!("{}", &scripts_root);
        dbg!(&readme_path);
        ExitCode::SUCCESS
    }
}
