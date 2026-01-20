mod error;
mod parser;

pub use error::GeddesError;
use parser::{parse_rasx, parse_raw, parse_xy, ParsedData};
use std::path::Path;
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

pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Pattern, GeddesError> {
    let path = path.as_ref();
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    let data = match ext.as_str() {
        "xy" | "xye" => parse_xy(path)?,
        "rasx" => parse_rasx(path)?,
        "raw" => parse_raw(path)?,
        _ => return Err(GeddesError::UnknownFormat),
    };
    
    Ok(data.into())
}
