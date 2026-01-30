//! # Geddes
//!
//! `geddes` is a library for loading and parsing various diffraction pattern file formats.
//! It supports common formats like `.raw`, `.rasx`, `.xrdml`, `.xy` / `.xye`, and `.csv`.

mod error;
mod parser;

#[cfg(feature = "python")]
mod python;

pub use error::Error;
use parser::{
    parse_bruker_raw, parse_csv, parse_gsas_raw, parse_rasx, parse_xrdml, parse_xy, ParsedPattern,
};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// Represents a diffraction pattern with position, intensity, and optional error.
#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pattern {
    /// The x-axis values (e.g., 2-theta or Q).
    pub x: Vec<f64>,
    /// The y-axis values (intensity).
    pub y: Vec<f64>,
    /// The uncertainty/error values, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<Vec<f64>>,
}

impl Pattern {
    /// Creates a new diffraction pattern, returning an error if lengths are inconsistent.
    pub fn new(x: Vec<f64>, y: Vec<f64>, e: Option<Vec<f64>>) -> Result<Self, Error> {
        if x.len() != y.len() {
            return Err(Error::Parse(
                "x and y must have the same length".into(),
            ));
        }
        if let Some(ref e_vec) = e {
            if e_vec.len() != x.len() {
                return Err(Error::Parse(
                    "e must have the same length as x and y".into(),
                ));
            }
        }
        Ok(Pattern { x, y, e })
    }
}

impl From<ParsedPattern> for Pattern {
    fn from(data: ParsedPattern) -> Self {
        Pattern {
            x: data.x,
            y: data.y,
            e: data.e,
        }
    }
}

/// Load a pattern from a file path.
///
/// Format is determined automatically by the file extension.
///
/// # Examples
///
/// ```no_run
/// use geddes::read;
///
/// let pattern = read("tests/data/xy/sample.xy").expect("Failed to load file");
/// println!("Loaded {} points", pattern.x.len());
/// ```
pub fn read<P: AsRef<Path>>(path: P) -> Result<Pattern, Error> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    read_reader(file, filename)
}

/// Load a pattern from any reader that implements Read + Seek.
///
/// This is useful for loading from bytes (using `Cursor<Vec<u8>>`) or other non-file sources,
/// which is particularly important for WASM environments.
///
/// # Arguments
///
/// * `reader` - The reader to read from. Must implement `Read` and `Seek`.
/// * `filename` - The name of the file (used to determine format via extension).
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use geddes::read_reader;
///
/// let data = b"10.0 100.0\n10.1 105.0";
/// let cursor = Cursor::new(data);
/// let pattern = read_reader(cursor, "data.xy").unwrap();
/// assert_eq!(pattern.x.len(), 2);
/// ```
pub fn read_reader<R: Read + Seek>(
    reader: R,
    filename: &str,
) -> Result<Pattern, Error> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut reader = reader;
    let data = match ext.as_str() {
        "raw" => {
            // Check for binary (Bruker) vs Text (GSAS)
            let mut buffer = [0u8; 1024];
            let bytes_read = reader.read(&mut buffer)?;
            reader.seek(SeekFrom::Start(0))?;

            // Simple heuristic: if we find null bytes, assume binary.
            let chunk = &buffer[..bytes_read];
            let is_binary = chunk.iter().any(|&b| b == 0);

            // GSAS usually starts with a title line or BANK, and is text.
            // Bruker binary usually has non-text bytes.

            if is_binary {
                parse_bruker_raw(reader)?
            } else {
                parse_gsas_raw(reader)?
            }
        }
        "rasx" => parse_rasx(reader)?,
        "xrdml" => parse_xrdml(reader)?,
        "xy" | "xye" => parse_xy(reader)?,
        "csv" => parse_csv(reader)?,
        _ => return Err(Error::UnknownFormat),
    };

    Ok(data.into())
}

/// Load a pattern from in-memory bytes with a filename hint.
pub fn read_bytes<B: AsRef<[u8]>>(bytes: B, filename: &str) -> Result<Pattern, Error> {
    let cursor = Cursor::new(bytes.as_ref());
    read_reader(cursor, filename)
}
