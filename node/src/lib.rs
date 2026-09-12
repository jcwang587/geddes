#![deny(clippy::all)]
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

#[napi(object)]
pub struct Pattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

#[napi(object)]
#[derive(Default)]
pub struct ReadOptions {
    pub index: Option<u32>,
    pub block: Option<String>,
}

impl From<geddes::Pattern> for Pattern {
    fn from(value: geddes::Pattern) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<ReadOptions> for geddes::ReadOptions {
    fn from(value: ReadOptions) -> Self {
        Self {
            index: value.index.unwrap_or(0) as usize,
            block: value.block,
        }
    }
}

fn to_napi_error(err: geddes::Error) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}

#[napi]
pub fn read(path: String, options: Option<ReadOptions>) -> napi::Result<Pattern> {
    geddes::read_with_options(path, &options.unwrap_or_default().into())
        .map(Into::into)
        .map_err(to_napi_error)
}

#[napi]
pub fn read_bytes(
    data: Buffer,
    filename: String,
    options: Option<ReadOptions>,
) -> napi::Result<Pattern> {
    geddes::read_bytes_with_options(
        data.as_ref(),
        &filename,
        &options.unwrap_or_default().into(),
    )
    .map(Into::into)
    .map_err(to_napi_error)
}
