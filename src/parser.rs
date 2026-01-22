use crate::error::GeddesError;
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
