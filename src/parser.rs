use crate::error::Error;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{BufRead, BufReader, Read, Seek};
use zip::ZipArchive;

/// Intermediate structure to hold parsed data before converting to the public Pattern struct.
#[derive(Debug)]
pub struct ParsedPattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub e: Option<Vec<f64>>,
}

/// Helper to parse x, y, and optional e from string parts.
fn parse_columns(parts: &[&str], x: &mut Vec<f64>, y: &mut Vec<f64>, e: &mut Vec<f64>) {
    if parts.len() >= 2 {
        if let (Ok(val_x), Ok(val_y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            x.push(val_x);
            y.push(val_y);
            if parts.len() >= 3 {
                if let Ok(val_e) = parts[2].parse::<f64>() {
                    e.push(val_e);
                }
            }
        }
    }
}

/// Parses standard XY files (two or three columns: x, y, [e]).
///
/// Ignores lines starting with '#' or '!'.
pub fn parse_xy<R: Read>(reader: R) -> Result<ParsedPattern, Error> {
    let reader = BufReader::new(reader);
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut e = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        parse_columns(&parts, &mut x, &mut y, &mut e);
    }

    let has_error = !e.is_empty() && e.len() == x.len();
    Ok(ParsedPattern {
        x,
        y,
        e: if has_error { Some(e) } else { None },
    })
}

/// Parses CSV files.
///
/// Supports comma or whitespace as delimiters.
pub fn parse_csv<R: Read>(reader: R) -> Result<ParsedPattern, Error> {
    let reader = BufReader::new(reader);
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut e = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        // Support both comma-separated and whitespace-separated CSV-like files.
        let parts: Vec<&str> = line
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        parse_columns(&parts, &mut x, &mut y, &mut e);
    }

    let has_error = !e.is_empty() && e.len() == x.len();
    Ok(ParsedPattern {
        x,
        y,
        e: if has_error { Some(e) } else { None },
    })
}

/// Parses Rigaku RASX files (zipped XML/text format).
///
/// Looks for a `Profile*.txt` file inside the archive.
pub fn parse_rasx<R: Read + Seek>(reader: R) -> Result<ParsedPattern, Error> {
    let mut archive = ZipArchive::new(reader)?;

    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
        .collect();

    // Prioritize Data0/Profile0.txt, or find any Profile*.txt
    let profile_name = names
        .iter()
        .find(|n| n.as_str() == "Data0/Profile0.txt")
        .or_else(|| {
            names
                .iter()
                .find(|n| n.contains("Profile") && n.ends_with(".txt"))
        })
        .ok_or_else(|| Error::FileNotFoundInArchive("Profile*.txt".to_string()))?;

    let file = archive.by_name(profile_name)?;
    let reader = BufReader::new(file);

    let mut x = Vec::new();
    let mut y = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(val_x), Ok(val_y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                x.push(val_x);
                y.push(val_y);
            }
        }
    }
    Ok(ParsedPattern { x, y, e: None })
}

/// Parses Panalytical XRDML files (XML-based).
///
/// Extracts the 2Theta start/end positions and the intensities list.
pub fn parse_xrdml<R: Read>(reader: R) -> Result<ParsedPattern, Error> {
    let reader = BufReader::new(reader);
    let mut xml = Reader::from_reader(reader);
    xml.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut intensities = Vec::new();
    let mut in_intensities = false;
    let mut in_positions_2theta = false;
    let mut capture_start = false;
    let mut capture_end = false;
    let mut start_pos: Option<f64> = None;
    let mut end_pos: Option<f64> = None;

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"positions" => {
                    in_positions_2theta = false;
                    for attr in e.attributes() {
                        let attr = attr.map_err(|err| {
                            Error::Parse(format!("XRDML attribute error: {err}"))
                        })?;
                        if attr.key.as_ref() == b"axis" {
                            let axis = attr
                                .unescape_value()
                                .map_err(|err| {
                                    Error::Parse(format!(
                                        "XRDML attribute decode error: {err}"
                                    ))
                                })?
                                .into_owned();
                            if axis == "2Theta" {
                                in_positions_2theta = true;
                            }
                        }
                    }
                }
                b"startPosition" => {
                    if in_positions_2theta {
                        capture_start = true;
                    }
                }
                b"endPosition" => {
                    if in_positions_2theta {
                        capture_end = true;
                    }
                }
                b"intensities" => {
                    in_intensities = true;
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                let text = e
                    .decode()
                    .map_err(|err| Error::Parse(format!("XRDML text decode error: {err}")))?;
                let text = text.trim();
                if text.is_empty() {
                    // Skip empty text nodes.
                } else if capture_start {
                    start_pos = Some(text.parse::<f64>().map_err(|_| {
                        Error::Parse("XRDML invalid 2Theta start position".into())
                    })?);
                } else if capture_end {
                    end_pos = Some(text.parse::<f64>().map_err(|_| {
                        Error::Parse("XRDML invalid 2Theta end position".into())
                    })?);
                } else if in_intensities {
                    for part in text.split_whitespace() {
                        if let Ok(value) = part.parse::<f64>() {
                            intensities.push(value);
                        }
                    }
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"positions" => {
                    in_positions_2theta = false;
                }
                b"startPosition" => {
                    capture_start = false;
                }
                b"endPosition" => {
                    capture_end = false;
                }
                b"intensities" => {
                    in_intensities = false;
                    if !intensities.is_empty() && start_pos.is_some() && end_pos.is_some() {
                        break;
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(Error::Parse(format!("XRDML parse error: {err}")));
            }
            _ => {}
        }
        buf.clear();
    }

    let start = start_pos
        .ok_or_else(|| Error::Parse("XRDML missing 2Theta start position".into()))?;
    let end =
        end_pos.ok_or_else(|| Error::Parse("XRDML missing 2Theta end position".into()))?;

    if intensities.is_empty() {
        return Err(Error::Parse("XRDML intensities not found".into()));
    }

    let mut x = Vec::with_capacity(intensities.len());
    if intensities.len() == 1 {
        x.push(start);
    } else {
        let step = (end - start) / (intensities.len() as f64 - 1.0);
        for i in 0..intensities.len() {
            x.push(start + (i as f64) * step);
        }
    }

    Ok(ParsedPattern {
        x,
        y: intensities,
        e: None,
    })
}

/// Parses GSAS RAW files.
///
/// Expects a `BANK` header line to determine start angle and step size.
pub fn parse_gsas_raw<R: Read>(reader: R) -> Result<ParsedPattern, Error> {
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let mut start = 0.0;
    let mut step = 0.0;

    let mut header_found = false;

    for line_res in lines.by_ref() {
        let line = line_res?;
        if line.starts_with("BANK") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // BANK 1 4941 494 CONST 1600.0 1.7 0.0 0.0 STD
            if parts.len() >= 7 {
                let start_raw = parts[5]
                    .parse::<f64>()
                    .map_err(|_| Error::Parse("Invalid start".into()))?;
                let step_raw = parts[6]
                    .parse::<f64>()
                    .map_err(|_| Error::Parse("Invalid step".into()))?;

                // GSAS standard: centidegrees
                start = start_raw / 100.0;
                step = step_raw / 100.0;
                header_found = true;
                break;
            }
        }
    }

    if !header_found {
        return Err(Error::Parse(
            "BANK header not found in RAW file".into(),
        ));
    }

    let mut y = Vec::new();

    for line in lines {
        let line = line?;
        if line.starts_with("BANK") {
            break;
        }
        let parts = line.split_whitespace();
        for part in parts {
            if let Ok(val) = part.parse::<f64>() {
                y.push(val);
            }
        }
    }

    // Generate x
    let mut x = Vec::with_capacity(y.len());
    for i in 0..y.len() {
        x.push(start + (i as f64) * step);
    }

    Ok(ParsedPattern { x, y, e: None })
}

/// Parses Bruker binary RAW files.
///
/// Uses heuristics to locate the intensity block and axis metadata.
///
/// Bruker RAW4 files are not fully documented and may store intensity points
/// either as contiguous `f32` values or as interleaved records near the file
/// tail (e.g. `f32 value` + `u32 status`).
pub fn parse_bruker_raw<R: Read>(mut reader: R) -> Result<ParsedPattern, Error> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    if buf.len() < 32 || !buf.starts_with(b"RAW") {
        return Err(Error::Parse(
            "Unsupported Bruker RAW header".into(),
        ));
    }

    let layout = find_bruker_data_layout(&buf)
        .ok_or_else(|| Error::Parse("Failed to locate Bruker RAW data block".into()))?;
    let count = layout.count;

    let count_usize = count as usize;
    let mut y = Vec::with_capacity(count_usize);
    for i in 0..count_usize {
        let off = layout.data_offset + i * layout.stride + layout.value_offset;
        let val = read_f32_le(&buf, off).ok_or_else(|| {
            Error::Parse("Bruker RAW intensity data truncated".into())
        })?;
        y.push(val as f64);
    }

    let mut x = Vec::with_capacity(count_usize);
    let count_offsets = find_bruker_count_offsets(&buf, count, layout.data_offset);
    if let Some((start, step)) = find_bruker_start_step(&buf, &count_offsets, count) {
        for i in 0..count_usize {
            x.push(start + step * (i as f64));
        }
    } else {
        // Some RAW4 variants do not expose a robust local start/step pair.
        // In that case we still return usable intensity points with index x.
        for i in 0..count_usize {
            x.push(i as f64);
        }
    }

    Ok(ParsedPattern { x, y, e: None })
}

#[derive(Debug, Clone, Copy)]
struct BrukerDataLayout {
    count: u32,
    data_offset: usize,
    stride: usize,
    value_offset: usize,
}

fn find_bruker_data_layout(buf: &[u8]) -> Option<BrukerDataLayout> {
    find_bruker_interleaved_tail_block(buf)
        .or_else(|| find_bruker_plain_f32_tail_block(buf))
}

fn find_bruker_plain_f32_tail_block(buf: &[u8]) -> Option<BrukerDataLayout> {
    let len = buf.len();
    let mut best: Option<BrukerDataLayout> = None;

    for off in 0..len.saturating_sub(4) {
        let count = read_u32_le(buf, off)?;
        if count < 10 || count > 5_000_000 {
            continue;
        }
        let data_len = (count as usize).checked_mul(4)?;
        if data_len > len {
            continue;
        }
        let data_offset = len - data_len;
        if data_offset <= off {
            continue;
        }

        let layout = BrukerDataLayout {
            count,
            data_offset,
            stride: 4,
            value_offset: 0,
        };
        if !bruker_data_layout_plausible(buf, layout) {
            continue;
        }

        match best {
            Some(best_layout) if count <= best_layout.count => {}
            _ => best = Some(layout),
        }
    }

    best
}

fn find_bruker_interleaved_tail_block(buf: &[u8]) -> Option<BrukerDataLayout> {
    const MIN_POINTS: usize = 32;
    const FLAG_MAX: u32 = 3;

    let len = buf.len();
    let mut best: Option<BrukerDataLayout> = None;

    for value_offset in [0usize, 4usize] {
        let companion_offset = if value_offset == 0 { 4usize } else { 0usize };
        let mut run = 0usize;

        while len >= (run + 1) * 8 {
            let rec_off = len - (run + 1) * 8;
            let flag = match read_u32_le(buf, rec_off + companion_offset) {
                Some(v) => v,
                None => break,
            };
            if flag > FLAG_MAX {
                break;
            }
            let val = match read_f32_le(buf, rec_off + value_offset) {
                Some(v) => v,
                None => break,
            };
            if !val.is_finite() || val.abs() > 1.0e9 {
                break;
            }
            run += 1;
        }

        if run < MIN_POINTS {
            continue;
        }

        let candidate = BrukerDataLayout {
            count: run as u32,
            data_offset: len - run * 8,
            stride: 8,
            value_offset,
        };
        if !bruker_data_layout_plausible(buf, candidate) {
            continue;
        }

        match best {
            Some(current) if candidate.count <= current.count => {}
            _ => best = Some(candidate),
        }
    }

    best
}

fn bruker_data_layout_plausible(buf: &[u8], layout: BrukerDataLayout) -> bool {
    let count = layout.count as usize;
    if count == 0 {
        return false;
    }
    let samples = 64.min(count);
    let marker_value = f32::from_bits(1);
    let mut marker_like = 0usize;
    let mut near_zero = 0usize;
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    for s in 0..samples {
        let idx = s * (count - 1) / (samples - 1).max(1);
        let off = layout.data_offset + idx * layout.stride + layout.value_offset;
        let val = match read_f32_le(buf, off) {
            Some(v) => v,
            None => return false,
        };
        if !val.is_finite() || val.abs() > 1.0e9 {
            return false;
        }
        if val == marker_value {
            marker_like += 1;
        }
        if val.abs() < 1.0e-30 {
            near_zero += 1;
        }
        min_val = min_val.min(val);
        max_val = max_val.max(val);
    }

    // Reject obvious marker-word streams (e.g. interleaved status words
    // accidentally interpreted as intensity values).
    if marker_like * 4 > samples {
        return false;
    }
    if near_zero * 3 > samples {
        return false;
    }
    if (max_val - min_val).abs() < 1.0e-6 {
        return false;
    }
    true
}

fn find_bruker_count_offsets(buf: &[u8], count: u32, search_end: usize) -> Vec<usize> {
    if search_end < 4 {
        return Vec::new();
    }
    let end = search_end.min(buf.len().saturating_sub(4));
    let mut offsets = Vec::new();

    for off in 0..=end {
        if read_u32_le(buf, off) == Some(count) {
            offsets.push(off);
        }
    }
    offsets
}

fn find_bruker_start_step(
    buf: &[u8],
    count_offsets: &[usize],
    count: u32,
) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64, f64)> = None;

    for &count_offset in count_offsets {
        if let Some(start_off) = count_offset.checked_sub(16) {
            if let (Some(start), Some(step)) =
                (read_f64_le(buf, start_off), read_f64_le(buf, start_off + 8))
            {
                if bruker_start_step_valid(start, step, count) {
                    let score = score_bruker_start_step(start, step, count) + 1000.0;
                    match best {
                        Some((_, _, best_score)) if score <= best_score => {}
                        _ => best = Some((start, step, score)),
                    }
                }
            }
        }

        // Fallback: scan a small window before the count for a plausible
        // (start, step) pair.
        let window_start = count_offset.saturating_sub(64);
        for off in window_start..count_offset {
            let start = match read_f64_le(buf, off) {
                Some(v) => v,
                None => continue,
            };
            let step = match read_f64_le(buf, off + 8) {
                Some(v) => v,
                None => continue,
            };
            if !bruker_start_step_valid(start, step, count) {
                continue;
            }
            let score = score_bruker_start_step(start, step, count);
            match best {
                Some((_, _, best_score)) if score <= best_score => {}
                _ => best = Some((start, step, score)),
            }
        }
    }

    best.map(|(start, step, _)| (start, step))
}

fn bruker_start_step_valid(start: f64, step: f64, count: u32) -> bool {
    if !start.is_finite() || !step.is_finite() || step <= 1.0e-6 || step > 10.0 {
        return false;
    }
    let n = count as f64;
    let end = start + step * if n > 1.0 { n - 1.0 } else { 0.0 };
    if !end.is_finite() || end < start {
        return false;
    }
    start >= -180.0 && end <= 360.0
}

fn score_bruker_start_step(start: f64, step: f64, count: u32) -> f64 {
    let span = step * ((count as f64 - 1.0).max(0.0));
    let mut score = span.min(360.0);
    if (0.0..=180.0).contains(&start) {
        score += 50.0;
    }
    if (1.0e-4..=0.5).contains(&step) {
        score += 25.0;
    }
    score
}

fn read_u32_le(buf: &[u8], offset: usize) -> Option<u32> {
    let bytes = buf.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_f32_le(buf: &[u8], offset: usize) -> Option<f32> {
    let bytes = buf.get(offset..offset + 4)?;
    Some(f32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_f64_le(buf: &[u8], offset: usize) -> Option<f64> {
    let bytes = buf.get(offset..offset + 8)?;
    Some(f64::from_le_bytes(bytes.try_into().ok()?))
}
