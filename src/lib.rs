//! # Geddes
//!
//! `geddes` is a library for loading and parsing various diffraction pattern file formats.
//! It supports common formats like `.xy`, `.csv`, `.rasx`, and `.raw`.

mod error;
mod parser;

pub use error::GeddesError;
use parser::{parse_bruker_raw, parse_csv, parse_gsas_raw, parse_rasx, parse_xy, ParsedData};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Represents a diffraction pattern with position, intensity, and optional error.
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
    /// Creates a new Pattern, returning an error if lengths are inconsistent.
    pub fn new(x: Vec<f64>, y: Vec<f64>, e: Option<Vec<f64>>) -> Result<Self, GeddesError> {
        if x.len() != y.len() {
            return Err(GeddesError::Parse(
                "x and y must have the same length".into(),
            ));
        }
        if let Some(ref e_vec) = e {
            if e_vec.len() != x.len() {
                return Err(GeddesError::Parse(
                    "e must have the same length as x and y".into(),
                ));
            }
        }
        Ok(Pattern { x, y, e })
    }
}

impl From<ParsedData> for Pattern {
    fn from(data: ParsedData) -> Self {
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
/// use geddes::load_file;
///
/// let pattern = load_file("tests/data/xy/sample.xy").expect("Failed to load file");
/// println!("Loaded {} points", pattern.x.len());
/// ```
pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Pattern, GeddesError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    load_from_reader(file, filename)
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
/// use geddes::load_from_reader;
///
/// let data = b"10.0 100.0\n10.1 105.0";
/// let cursor = Cursor::new(data);
/// let pattern = load_from_reader(cursor, "data.xy").unwrap();
/// assert_eq!(pattern.x.len(), 2);
/// ```
pub fn load_from_reader<R: Read + Seek>(reader: R, filename: &str) -> Result<Pattern, GeddesError> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut reader = reader;
    let data = match ext.as_str() {
        "xy" | "xye" => parse_xy(reader)?,
        "csv" => parse_csv(reader)?,
        "rasx" => parse_rasx(reader)?,
        "raw" => {
            // Check for binary (Bruker) vs Text (GSAS)
            let mut buffer = [0u8; 1024];
            let bytes_read = reader.read(&mut buffer)?;
            reader.seek(SeekFrom::Start(0))?;

            // Simple heuristic: if we find null bytes or excessive non-printable chars, assume binary.
            // Or if we see "BANK" in the first chunk, assume GSAS.
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
        _ => return Err(GeddesError::UnknownFormat),
    };

    Ok(data.into())
}
