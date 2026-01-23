use crate::error::GeddesError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{BufRead, BufReader, Read, Seek};
use zip::ZipArchive;

/// Intermediate structure to hold parsed data before converting to the public Pattern struct.
#[derive(Debug)]
pub struct ParsedData {
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
pub fn parse_xy<R: Read>(reader: R) -> Result<ParsedData, GeddesError> {
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
    Ok(ParsedData {
        x,
        y,
        e: if has_error { Some(e) } else { None },
    })
}

/// Parses CSV files.
///
/// Supports comma or whitespace as delimiters.
pub fn parse_csv<R: Read>(reader: R) -> Result<ParsedData, GeddesError> {
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
    Ok(ParsedData {
        x,
        y,
        e: if has_error { Some(e) } else { None },
    })
}

/// Parses Rigaku RASX files (zipped XML/text format).
///
/// Looks for a `Profile*.txt` file inside the archive.
pub fn parse_rasx<R: Read + Seek>(reader: R) -> Result<ParsedData, GeddesError> {
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
        .ok_or_else(|| GeddesError::FileNotFoundInArchive("Profile*.txt".to_string()))?;

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
    Ok(ParsedData { x, y, e: None })
}

/// Parses Panalytical XRDML files (XML-based).
///
/// Extracts the 2Theta start/end positions and the intensities list.
pub fn parse_xrdml<R: Read>(reader: R) -> Result<ParsedData, GeddesError> {
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
                            GeddesError::Parse(format!("XRDML attribute error: {err}"))
                        })?;
                        if attr.key.as_ref() == b"axis" {
                            let axis = attr
                                .unescape_value()
                                .map_err(|err| {
                                    GeddesError::Parse(format!(
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
                    .map_err(|err| GeddesError::Parse(format!("XRDML text decode error: {err}")))?;
                let text = text.trim();
                if text.is_empty() {
                    // Skip empty text nodes.
                } else if capture_start {
                    start_pos = Some(text.parse::<f64>().map_err(|_| {
                        GeddesError::Parse("XRDML invalid 2Theta start position".into())
                    })?);
                } else if capture_end {
                    end_pos = Some(text.parse::<f64>().map_err(|_| {
                        GeddesError::Parse("XRDML invalid 2Theta end position".into())
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
                return Err(GeddesError::Parse(format!("XRDML parse error: {err}")));
            }
            _ => {}
        }
        buf.clear();
    }

    let start = start_pos
        .ok_or_else(|| GeddesError::Parse("XRDML missing 2Theta start position".into()))?;
    let end =
        end_pos.ok_or_else(|| GeddesError::Parse("XRDML missing 2Theta end position".into()))?;

    if intensities.is_empty() {
        return Err(GeddesError::Parse("XRDML intensities not found".into()));
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

    Ok(ParsedData {
        x,
        y: intensities,
        e: None,
    })
}

/// Parses GSAS RAW files.
///
/// Expects a `BANK` header line to determine start angle and step size.
pub fn parse_gsas_raw<R: Read>(reader: R) -> Result<ParsedData, GeddesError> {
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
                    .map_err(|_| GeddesError::Parse("Invalid start".into()))?;
                let step_raw = parts[6]
                    .parse::<f64>()
                    .map_err(|_| GeddesError::Parse("Invalid step".into()))?;

                // GSAS standard: centidegrees
                start = start_raw / 100.0;
                step = step_raw / 100.0;
                header_found = true;
                break;
            }
        }
    }

    if !header_found {
        return Err(GeddesError::Parse(
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

    Ok(ParsedData { x, y, e: None })
}

/// Parses Bruker binary RAW files.
///
/// (Currently a placeholder returning an error)
pub fn parse_bruker_raw<R: Read>(_reader: R) -> Result<ParsedData, GeddesError> {
    Err(GeddesError::Parse(
        "Bruker binary RAW format not yet supported".into(),
    ))
}
