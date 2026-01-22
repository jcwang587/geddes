use crate::{load_file, load_from_reader, GeddesError, Pattern};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io::Cursor;

/// Convert a Rust library error into a Python exception.
fn to_py_err(err: GeddesError) -> PyErr {
    match err {
        GeddesError::Io(err) => PyIOError::new_err(err.to_string()),
        GeddesError::Zip(err) => PyValueError::new_err(err.to_string()),
        GeddesError::Parse(msg) => PyValueError::new_err(msg),
        GeddesError::UnknownFormat => PyValueError::new_err("Unknown format"),
        GeddesError::FileNotFoundInArchive(name) => {
            PyValueError::new_err(format!("File not found in archive: {}", name))
        }
    }
}

/// Load a diffraction pattern from a file path.
#[pyfunction(name = "load_file")]
fn load_file_py(path: &str) -> PyResult<Pattern> {
    load_file(path).map_err(to_py_err)
}

/// Load a diffraction pattern from raw bytes with a filename hint.
#[pyfunction]
fn load_bytes(data: &Bound<'_, PyBytes>, filename: &str) -> PyResult<Pattern> {
    let cursor = Cursor::new(data.as_bytes());
    load_from_reader(cursor, filename).map_err(to_py_err)
}

/// Python module definition for the `geddes` extension.
#[pymodule]
fn geddes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pattern>()?;
    m.add_function(wrap_pyfunction!(load_file_py, m)?)?;
    m.add_function(wrap_pyfunction!(load_bytes, m)?)?;
    Ok(())
}
