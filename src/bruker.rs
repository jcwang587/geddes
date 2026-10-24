//! Bruker RAW3 and RAW4: decode the declared range headers and data strides.
//!
//! Field offsets are format facts cross-checked against the Rietx format notes
//! and its independently packed examples. In both versions the first float32
//! of a data record is intensity; RAW3 can additionally record measured 2θ.

use crate::parser::{axis_key, ParsedPattern};
use crate::Error;

const MAX_RANGES: usize = 4096;
const MAX_HEADER: usize = 1 << 20;

struct Range {
    start: f64,
    step: f64,
    points: usize,
    stride: usize,
    data_start: usize,
    data_end: usize,
    measured_two_theta: bool,
    axis: Result<(), String>,
}

fn raw_error(message: impl Into<String>) -> Error {
    Error::parse(format!("Bruker RAW: {}", message.into()))
}

fn bytes_at(bytes: &[u8], at: usize, length: usize) -> Result<&[u8], Error> {
    let end = at
        .checked_add(length)
        .ok_or_else(|| raw_error("offset overflow"))?;
    bytes.get(at..end).ok_or_else(|| {
        raw_error(format!(
            "truncated record at offset {at}: needs {length} bytes, file has {}",
            bytes.len()
        ))
    })
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(
        bytes_at(bytes, at, 4)?.try_into().unwrap(),
    ))
}

fn f64_at(bytes: &[u8], at: usize) -> Result<f64, Error> {
    Ok(f64::from_le_bytes(
        bytes_at(bytes, at, 8)?.try_into().unwrap(),
    ))
}

fn text_at(bytes: &[u8], at: usize, length: usize) -> Result<String, Error> {
    Ok(bytes_at(bytes, at, length)?
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| char::from(b))
        .collect::<String>()
        .trim()
        .to_owned())
}

fn scan_type_axis(name: &str) -> Result<(), String> {
    match axis_key(name).as_str() {
        "lockedcoupled" | "unlockedcoupled" | "unlockedcoupledhrxrd" | "detectorscan" => Ok(()),
        _ => Err(format!(
            "scan type {name:?} does not establish a 2theta axis; export a coupled 2theta scan"
        )),
    }
}

fn segment(bytes: &[u8], at: usize, end: usize) -> Result<(u32, usize), Error> {
    if end.saturating_sub(at) < 8 {
        return Err(raw_error("truncated segment header"));
    }
    let kind = u32_at(bytes, at)?;
    let length = u32_at(bytes, at + 4)? as usize;
    if length < 8 || length > end - at {
        return Err(raw_error(format!(
            "segment type {kind} at offset {at} declares invalid length {length}"
        )));
    }
    Ok((kind, length))
}

fn range_end(
    bytes: &[u8],
    data_start: usize,
    points: usize,
    stride: usize,
) -> Result<usize, Error> {
    if points == 0 {
        return Err(raw_error("range has no points"));
    }
    let size = points
        .checked_mul(stride)
        .ok_or_else(|| raw_error("data size overflow"))?;
    bytes_at(bytes, data_start, size)?;
    Ok(data_start + size)
}

fn check_axis_numbers(start: f64, step: f64) -> Result<(), Error> {
    if !start.is_finite() || !step.is_finite() || step == 0.0 {
        return Err(raw_error("range start and nonzero step must be finite"));
    }
    Ok(())
}

fn raw4_range(bytes: &[u8], marker: usize) -> Result<Range, Error> {
    bytes_at(bytes, marker, 160)?;
    let scan_type = text_at(bytes, marker + 32, 24)?;
    let start = f64_at(bytes, marker + 72)?;
    let step = f64_at(bytes, marker + 80)?;
    check_axis_numbers(start, step)?;
    let points = u32_at(bytes, marker + 88)? as usize;
    let stride = u32_at(bytes, marker + 136)? as usize;
    let header_size = u32_at(bytes, marker + 140)? as usize;
    if stride < 4 || !stride.is_multiple_of(4) {
        return Err(raw_error(format!("invalid RAW4 data-record size {stride}")));
    }
    if header_size > MAX_HEADER {
        return Err(raw_error("RAW4 drive header exceeds 1 MiB"));
    }
    let header = marker + 160;
    bytes_at(bytes, header, header_size)?;
    let data_start = header + header_size;
    let data_end = range_end(bytes, data_start, points, stride)?;

    // Two independent header statements identify a scanned drive: its flag
    // must be set and its position must agree with the range start.
    let mut cursor = header;
    let mut candidates = Vec::new();
    while cursor < data_start {
        let (kind, length) = segment(bytes, cursor, data_start)?;
        if kind == 50 {
            if length < 64 {
                return Err(raw_error("RAW4 drive record is shorter than 64 bytes"));
            }
            let flag = u32_at(bytes, cursor + 8)?;
            let position = f64_at(bytes, cursor + 56)?;
            if flag != 0 && (position - start).abs() <= 1e-6 + 1e-5 * start.abs() {
                candidates.push(text_at(bytes, cursor + 12, 24)?);
            }
        }
        cursor += length;
    }
    let axis = if candidates.len() == 1 {
        let name = &candidates[0];
        match axis_key(name).as_str() {
            "2theta" | "twotheta" => Ok(()),
            _ => Err(format!(
                "scanned drive {name:?} is not a recognized 2theta axis"
            )),
        }
    } else {
        scan_type_axis(&scan_type)
    };

    Ok(Range {
        start,
        step,
        points,
        stride,
        data_start,
        data_end,
        measured_two_theta: false,
        axis,
    })
}

fn raw4_ranges(bytes: &[u8]) -> Result<Vec<Range>, Error> {
    bytes_at(bytes, 0, 61)?;
    let mut cursor = 61;
    let mut ranges = Vec::new();
    while cursor < bytes.len() {
        let kind = u32_at(bytes, cursor)?;
        if kind == 0 || kind == 160 {
            if ranges.len() >= MAX_RANGES {
                return Err(raw_error("too many RAW4 ranges"));
            }
            let found = raw4_range(bytes, cursor)?;
            cursor = found.data_end;
            ranges.push(found);
        } else {
            let (_, length) = segment(bytes, cursor, bytes.len())?;
            cursor += length;
        }
    }
    if ranges.is_empty() {
        return Err(raw_error("RAW4 file has no measurement ranges"));
    }
    Ok(ranges)
}

fn raw3_ranges(bytes: &[u8]) -> Result<Vec<Range>, Error> {
    bytes_at(bytes, 0, 712)?;
    let count = u32_at(bytes, 12)? as usize;
    if !(1..=MAX_RANGES).contains(&count) {
        return Err(raw_error(format!("invalid RAW3 range count {count}")));
    }
    let mut cursor = 712;
    let mut ranges = Vec::with_capacity(count);
    for _ in 0..count {
        let header_size = u32_at(bytes, cursor)? as usize;
        if !(260..=MAX_HEADER).contains(&header_size) {
            return Err(raw_error(format!(
                "invalid RAW3 range-header size {header_size}"
            )));
        }
        bytes_at(bytes, cursor, header_size)?;
        let points = u32_at(bytes, cursor + 4)? as usize;
        let start = f64_at(bytes, cursor + 16)?;
        let step = f64_at(bytes, cursor + 176)?;
        check_axis_numbers(start, step)?;
        let scan_type = u32_at(bytes, cursor + 196)?;
        let varying = u32_at(bytes, cursor + 248)?;
        let stride = u32_at(bytes, cursor + 252)? as usize;
        let extra = u32_at(bytes, cursor + 256)? as usize;
        let expected_stride = 4 + 8 * varying.count_ones() as usize;
        if varying > i32::MAX as u32 || stride != expected_stride {
            return Err(raw_error(format!(
                "RAW3 data-record size {stride} disagrees with varying-parameter bits {varying:#x} (expected {expected_stride})"
            )));
        }
        if extra > MAX_HEADER {
            return Err(raw_error("invalid RAW3 extra-record length"));
        }
        let data_start = (cursor + header_size)
            .checked_add(extra)
            .ok_or_else(|| raw_error("RAW3 data offset overflow"))?;
        let data_end = range_end(bytes, data_start, points, stride)?;
        let axis = match scan_type {
            0 | 1 | 2 | 20 => Ok(()),
            _ => Err(format!(
                "RAW3 scan type {scan_type} does not establish a 2theta axis"
            )),
        };
        ranges.push(Range {
            start,
            step,
            points,
            stride,
            data_start,
            data_end,
            measured_two_theta: varying & 1 != 0,
            axis,
        });
        cursor = data_end;
    }
    // Real RAW3 series can end with zero-filled alignment padding. Nonzero
    // bytes after the declared ranges indicate a wrong layout or truncation.
    if bytes[cursor..].iter().any(|&b| b != 0) {
        return Err(raw_error(
            "unaccounted nonzero bytes after the declared RAW3 ranges",
        ));
    }
    Ok(ranges)
}

/// Read one RAW3 or RAW4 range. The dispatcher routes only RAW signatures
/// here, so any other signature is version 1 or 2.
pub(crate) fn parse_raw(bytes: &[u8], index: usize) -> Result<ParsedPattern, Error> {
    let ranges = if bytes.starts_with(b"RAW1.01") {
        raw3_ranges(bytes)?
    } else if bytes.starts_with(b"RAW4.00") {
        raw4_ranges(bytes)?
    } else {
        return Err(raw_error(
            "versions 1 and 2 are unsupported; export RAW3, RAW4, UXD, or XY",
        ));
    };
    let found = ranges.get(index).ok_or_else(|| {
        raw_error(format!(
            "scan {index} is out of range; file has {} scans",
            ranges.len()
        ))
    })?;
    if let Err(message) = &found.axis {
        return Err(raw_error(message.clone()));
    }
    let mut x = Vec::with_capacity(found.points);
    let mut y = Vec::with_capacity(found.points);
    for (i, record) in bytes[found.data_start..found.data_end]
        .chunks_exact(found.stride)
        .enumerate()
    {
        let intensity = f32::from_le_bytes(record[..4].try_into().unwrap()) as f64;
        let angle = if found.measured_two_theta {
            f64::from_le_bytes(record[4..12].try_into().unwrap())
        } else {
            found.start + found.step * i as f64
        };
        if !intensity.is_finite() || !angle.is_finite() {
            return Err(raw_error(format!(
                "nonfinite angle or intensity in point {i}"
            )));
        }
        x.push(angle);
        y.push(intensity);
    }
    Ok(ParsedPattern { x, y })
}
