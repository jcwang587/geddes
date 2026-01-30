use crate::{read, read_reader, Pattern, Error};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io::Cursor;

/// Convert a Rust library error into a Python exception.
fn to_py_err(err: Error) -> PyErr {
    match err {
        Error::Io(err) => PyIOError::new_err(err.to_string()),
        Error::Zip(err) => PyValueError::new_err(err.to_string()),
        Error::Parse(msg) => PyValueError::new_err(msg),
        Error::UnknownFormat => PyValueError::new_err("Unknown format"),
        Error::FileNotFoundInArchive(name) => {
            PyValueError::new_err(format!("File not found in archive: {}", name))
        }
    }
}

/// Load a pattern from a file path.
#[pyfunction(name = "read")]
fn read_py(path: &str) -> PyResult<Pattern> {
    read(path).map_err(to_py_err)
}

/// Load a pattern from raw bytes with a filename hint.
#[pyfunction]
fn read_bytes(
    data: &Bound<'_, PyBytes>,
    filename: &str,
) -> PyResult<Pattern> {
    let cursor = Cursor::new(data.as_bytes());
    read_reader(cursor, filename).map_err(to_py_err)
}

/// Python module definition for the `geddes` extension.
#[pymodule]
fn geddes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pattern>()?;
    m.add_function(wrap_pyfunction!(read_py, m)?)?;
    m.add_function(wrap_pyfunction!(read_bytes, m)?)?;
    Ok(())
}
