mod error;
mod parser;

pub use error::GeddesError;
use parser::{parse_rasx, parse_raw, parse_xy, ParsedData};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Pattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<Vec<f64>>,
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
pub fn load_from_reader<R: Read + Seek>(reader: R, filename: &str) -> Result<Pattern, GeddesError> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let data = match ext.as_str() {
        "xy" | "xye" => parse_xy(reader)?,
        "rasx" => parse_rasx(reader)?,
        "raw" => parse_raw(reader)?,
        _ => return Err(GeddesError::UnknownFormat),
    };
    
    Ok(data.into())
}
