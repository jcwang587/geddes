#![deny(clippy::all)]

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

#[napi(object)]
pub struct Pattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub e: Option<Vec<f64>>,
}

impl From<geddes::Pattern> for Pattern {
    fn from(value: geddes::Pattern) -> Self {
        Self {
            x: value.x,
            y: value.y,
            e: value.e,
        }
    }
}

fn to_napi_error(err: geddes::Error) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}

#[napi]
pub fn read(path: String) -> napi::Result<Pattern> {
    geddes::read(path).map(Into::into).map_err(to_napi_error)
}

#[napi]
pub fn read_bytes(data: Buffer, filename: String) -> napi::Result<Pattern> {
    geddes::read_bytes(data.as_ref(), &filename)
        .map(Into::into)
        .map_err(to_napi_error)
}
