use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeddesError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unknown format")]
    UnknownFormat,
    #[error("File not found in archive: {0}")]
    FileNotFoundInArchive(String),
}
