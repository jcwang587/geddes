//! Load XRD patterns as two arrays: 2θ in degrees (`x`) and intensity (`y`).
//!
//! Supports ASCII columns, GSAS, Rigaku RAS/RASX, Bruker RAW/UXD/BRML,
//! PANalytical XRDML, FIT2D CHI and powder CIF. No refinement or metadata API.

mod bruker;
mod error;
mod parser;
#[cfg(feature = "python")]
mod python;
mod text;
mod xml;

pub use error::Error;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use std::path::Path;

/// One diffraction pattern. Extra file columns are deliberately not returned.
#[cfg_attr(feature = "python", pyclass(get_all, skip_from_py_object))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pattern {
    /// Diffraction angle 2θ, in degrees, strictly increasing.
    pub x: Vec<f64>,
    /// Intensity in the file's units (counts or counts per second).
    pub y: Vec<f64>,
}

impl Pattern {
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Result<Self, Error> {
        if x.len() != y.len() {
            return Err(Error::Parse("x and y must have the same length".into()));
        }
        if x.is_empty() {
            return Err(Error::Parse("pattern contains no points".into()));
        }
        if x.iter().any(|v| !v.is_finite()) || x.windows(2).any(|w| w[0] >= w[1]) {
            return Err(Error::Parse(
                "x values must be finite and strictly increasing".into(),
            ));
        }
        if y.iter().any(|v| !v.is_finite()) {
            return Err(Error::Parse("y values must be finite".into()));
        }
        Ok(Self { x, y })
    }

    fn from_parsed(mut data: parser::ParsedPattern) -> Result<Self, Error> {
        if data.x.len() > 1 && data.x.windows(2).all(|w| w[0] > w[1]) {
            data.x.reverse();
            data.y.reverse();
        }
        Self::new(data.x, data.y)
    }
}

/// Choose one pattern without adding metadata to the returned arrays.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Zero-based scan/bank index for GSAS, RAW, RAS, RASX, UXD, XRDML and BRML.
    pub scan: usize,
    /// Substring identifying a powder CIF data block. First matching profile by default.
    pub block: Option<String>,
}

/// Load the first pattern in a file, detecting its content before its suffix.
/// ```no_run
/// let pattern = geddes::read("sample.xrdml")?;
/// println!("{} points", pattern.x.len());
/// # Ok::<(), geddes::Error>(())
/// ```
pub fn read<P: AsRef<Path>>(path: P) -> Result<Pattern, Error> {
    read_with_options(path, &ReadOptions::default())
}

pub fn read_with_options<P: AsRef<Path>>(path: P, options: &ReadOptions) -> Result<Pattern, Error> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)?;
    read_bytes_with_options(&bytes, path.to_str().unwrap_or(""), options)
}

/// Load from a seekable stream. `filename` is a hint when content is ambiguous.
pub fn from_reader<R: Read + Seek>(reader: R, filename: &str) -> Result<Pattern, Error> {
    from_reader_with_options(reader, filename, &ReadOptions::default())
}

pub fn from_reader_with_options<R: Read + Seek>(
    mut reader: R,
    filename: &str,
    options: &ReadOptions,
) -> Result<Pattern, Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    read_bytes_with_options(&bytes, filename, options)
}

/// Load bytes without opening a file. Extra columns, including uncertainties, are ignored.
pub fn read_bytes<B: AsRef<[u8]>>(bytes: B, filename: &str) -> Result<Pattern, Error> {
    read_bytes_with_options(bytes, filename, &ReadOptions::default())
}

pub fn read_bytes_with_options<B: AsRef<[u8]>>(
    bytes: B,
    filename: &str,
    options: &ReadOptions,
) -> Result<Pattern, Error> {
    Pattern::from_parsed(parser::parse(bytes.as_ref(), filename, options)?)
}
