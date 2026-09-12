use crate::{read_bytes_with_options, read_with_options, Error, Pattern, ReadOptions};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn to_py_err(err: Error) -> PyErr {
    match err {
        Error::Io(err) => PyIOError::new_err(err.to_string()),
        other => PyValueError::new_err(other.to_string()),
    }
}

#[pymethods]
impl Pattern {
    #[new]
    fn py_new(x: Vec<f64>, y: Vec<f64>) -> PyResult<Self> {
        Pattern::new(x, y).map_err(to_py_err)
    }
}

/// Load one XRD pattern as x (2theta degrees) and y (intensity).
#[pyfunction(name = "read", signature = (path, *, index=0, block=None))]
fn read_py(path: &str, index: usize, block: Option<String>) -> PyResult<Pattern> {
    read_with_options(path, &ReadOptions { index, block }).map_err(to_py_err)
}

/// Load bytes using content detection and a filename hint.
#[pyfunction(signature = (data, filename, *, index=0, block=None))]
fn read_bytes(
    data: &Bound<'_, PyBytes>,
    filename: &str,
    index: usize,
    block: Option<String>,
) -> PyResult<Pattern> {
    read_bytes_with_options(data.as_bytes(), filename, &ReadOptions { index, block })
        .map_err(to_py_err)
}

#[pymodule]
fn geddes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pattern>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(read_py, m)?)?;
    m.add_function(wrap_pyfunction!(read_bytes, m)?)?;
    Ok(())
}
