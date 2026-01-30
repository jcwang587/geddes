use std::io;
use thiserror::Error as ThisError;

/// Errors that can occur when loading or parsing data files.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An input/output error occurred.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// An error occurred while processing a ZIP archive (e.g. .rasx files).
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// The file content could not be parsed correctly.
    #[error("Parse error: {0}")]
    Parse(String),

    /// The file format is not recognized.
    #[error("Unknown format")]
    UnknownFormat,

    /// A required file was not found within the archive.
    #[error("File not found in archive: {0}")]
    FileNotFoundInArchive(String),
}
