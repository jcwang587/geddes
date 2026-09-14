//! Text diffraction formats. Only measured positions and intensities are retained.
use crate::{
    error::Error,
    parser::{decode, ParsedPattern},
};
use std::collections::HashMap;

fn invalid(message: impl Into<String>) -> Error {
    Error::Parse(message.into())
}

fn number(token: &str) -> Result<f64, Error> {
    let value = token
        .parse::<f64>()
        .or_else(|_| {
            if token.contains(['d', 'D']) {
                token.replace(['d', 'D'], "E").parse::<f64>()
            } else {
                token.parse::<f64>()
            }
        })
        .map_err(|_| invalid(format!("Invalid numeric value {token:?}")))?;
    if !value.is_finite() {
        return Err(invalid(format!("Non-finite numeric value {token:?}")));
    }
    Ok(value)
}

fn pattern(x: Vec<f64>, y: Vec<f64>, format: &str) -> Result<ParsedPattern, Error> {
    if x.is_empty() || x.len() != y.len() {
        return Err(invalid(format!("{format}: no complete x/y pattern found")));
    }
    Ok(ParsedPattern { x, y })
}

fn fields(line: &str) -> impl Iterator<Item = &str> {
    line.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
}

fn xy_row(line: &str, format: &str) -> Result<(f64, f64), Error> {
    let mut parts = fields(line);
    let x = parts
        .next()
        .ok_or_else(|| invalid(format!("{format}: missing position")))?;
    let y = parts
        .next()
        .ok_or_else(|| invalid(format!("{format}: missing intensity")))?;
    Ok((number(x)?, number(y)?))
}

pub(crate) fn parse_xy(bytes: &[u8]) -> Result<ParsedPattern, Error> {
    let mut x = Vec::new();
    let mut y = Vec::new();
    let text = decode(bytes)?;
    for line in text.lines() {
        let row = line.trim().trim_start_matches('\u{feff}');
        if row.contains('\0') {
            return Err(invalid("Binary data supplied to an ASCII pattern reader"));
        }
        if !row.is_empty() && !row.starts_with(['#', '!', '\'', '/', ';']) {
            let mut parts = fields(row);
            if let (Some(a), Some(b)) = (parts.next(), parts.next()) {
                // As in conventional XY readers, prose headers are ignored.
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    if !a.is_finite() || !b.is_finite() {
                        return Err(invalid("ASCII pattern contains a non-finite value"));
                    }
                    x.push(a);
                    y.push(b);
                }
            }
        }
    }
    pattern(x, y, "ASCII")
}

pub(crate) fn parse_gsas(bytes: &[u8], scan: usize) -> Result<ParsedPattern, Error> {
    let text = decode(bytes)?;
    let lines: Vec<&str> = text.lines().collect();
    let banks: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| (line.split_whitespace().next() == Some("BANK")).then_some(i))
        .collect();
    let &start_line = banks.get(scan).ok_or_else(|| {
        invalid(format!(
            "GSAS: bank index {scan} is out of range ({} banks)",
            banks.len()
        ))
    })?;
    let header: Vec<&str> = lines[start_line].split_whitespace().collect();
    if header.len() < 5 {
        return Err(invalid("GSAS: incomplete BANK header"));
    }
    let bintype = header[4].to_ascii_uppercase();
    if !matches!(bintype.as_str(), "CONS" | "CONST") {
        return Err(invalid(format!(
            "GSAS: {bintype} binning does not provide a supported 2theta axis"
        )));
    }
    if header.len() < 7 {
        return Err(invalid("GSAS: CONS/CONST requires a start and step"));
    }
    let count = header[2]
        .parse::<usize>()
        .map_err(|_| invalid("GSAS: invalid channel count"))?;
    if count == 0 {
        return Err(invalid("GSAS: zero channels"));
    }
    let start = number(header[5])? / 100.0;
    let step = number(header[6])? / 100.0;
    let mut flag = "STD";
    for token in &header[7..] {
        if token.starts_with(['#', '!']) {
            break;
        }
        if token
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
        {
            flag = token;
            break;
        }
        number(token)?;
    }
    let flag = flag.to_ascii_uppercase();
    if !matches!(flag.as_str(), "STD" | "ESD" | "FXYE") {
        return Err(invalid(format!("GSAS: unsupported record type {flag}")));
    }
    let end_line = banks.get(scan + 1).copied().unwrap_or(lines.len());
    let body: Vec<&str> = lines[start_line + 1..end_line]
        .iter()
        .copied()
        .filter(|l| {
            let s = l.trim();
            !s.is_empty() && !s.starts_with(['#', '!'])
        })
        .collect();
    let mut x = Vec::new();
    let mut y = Vec::new();
    if flag == "FXYE" {
        for line in body {
            let mut row = line.split_whitespace();
            let a = row
                .next()
                .ok_or_else(|| invalid("GSAS FXYE: missing position"))?;
            let b = row
                .next()
                .ok_or_else(|| invalid("GSAS FXYE: missing intensity"))?;
            let esd = row
                .next()
                .ok_or_else(|| invalid("GSAS FXYE: missing third column"))?;
            if row.next().is_some() {
                return Err(invalid(
                    "GSAS FXYE: each record must have exactly three columns",
                ));
            }
            x.push(number(a)? / 100.0);
            y.push(number(b)?);
            number(esd)?;
        }
        // APS 11-BM exports can carry a final endpoint beyond NCHAN. Keep
        // all explicit FXYE rows, as their positions are self-contained.
        if y.len() < count {
            return Err(invalid(format!(
                "GSAS FXYE: BANK declares {count} points but {} were read",
                y.len()
            )));
        }
    } else {
        if step == 0.0 {
            return Err(invalid("GSAS: step must be nonzero"));
        }
        let mut values = Vec::new();
        for line in body {
            if flag == "ESD" {
                // ESD has ten 8-character fields per 80-character record. A
                // full-width number may touch its neighbor with no whitespace.
                if !line.is_ascii() {
                    return Err(invalid("GSAS ESD: non-ASCII numeric record"));
                }
                if line.len() >= 16 && line.len() % 8 == 0 {
                    for chunk in line.as_bytes().chunks(8) {
                        let field = std::str::from_utf8(chunk).unwrap().trim();
                        if !field.is_empty() {
                            values.push(number(field)?);
                        }
                    }
                } else {
                    // Some exporters use free-format ESD pairs. Fixed-width
                    // records with trailing spaces removed still parse first.
                    let fixed: Result<Vec<f64>, Error> = line
                        .as_bytes()
                        .chunks(8)
                        .filter_map(|p| {
                            let token = std::str::from_utf8(p).unwrap().trim();
                            (!token.is_empty()).then(|| number(token))
                        })
                        .collect();
                    match fixed {
                        Ok(v) if v.len() % 2 == 0 => values.extend(v),
                        _ => {
                            let v: Result<Vec<_>, _> =
                                line.split_whitespace().map(number).collect();
                            let v = v?;
                            if v.len() % 2 != 0 {
                                return Err(invalid("GSAS ESD: incomplete intensity/esd pair"));
                            }
                            values.extend(v);
                        }
                    }
                }
            } else {
                // Uncompressed STD records reserve the first two characters
                // of each 8-character field. Do not mistake a repeat count
                // for an intensity; compressed STD is outside the reference
                // reader's supported layouts and is explicitly rejected.
                if line.is_ascii() && line.len() >= 8 && line.len() % 8 == 0 {
                    let chunks: Vec<&[u8]> = line.as_bytes().chunks(8).collect();
                    let positional = chunks.iter().all(|p| {
                        let prefix = std::str::from_utf8(&p[..2]).unwrap().trim();
                        let value = std::str::from_utf8(&p[2..]).unwrap().trim();
                        (prefix.is_empty() || prefix.parse::<u8>().is_ok())
                            && !value.is_empty()
                            && number(value).is_ok()
                    });
                    if positional {
                        for chunk in chunks {
                            let repeat = std::str::from_utf8(&chunk[..2]).unwrap().trim();
                            if !repeat.is_empty() && repeat != "0" && repeat != "1" {
                                return Err(invalid(
                                    "GSAS STD: compressed repeat-count records are unsupported",
                                ));
                            }
                            values.push(number(std::str::from_utf8(&chunk[2..]).unwrap().trim())?);
                        }
                        continue;
                    }
                }
                values.extend(
                    line.split_whitespace()
                        .map(number)
                        .collect::<Result<Vec<_>, _>>()?,
                );
            }
        }
        let width = if flag == "ESD" { 2 } else { 1 };
        if values.len() % width != 0 || values.len() / width < count {
            return Err(invalid(format!(
                "GSAS {flag}: data is shorter than the declared {count} channels"
            )));
        }
        // The last legacy record may be padded with zero fields to 80 columns.
        if values[count * width..].iter().any(|v| *v != 0.0) || values.len() - count * width >= 10 {
            return Err(invalid(format!(
                "GSAS {flag}: unexpected data beyond the declared channel count"
            )));
        }
        for i in 0..count {
            x.push(start + i as f64 * step);
            y.push(values[i * width]);
        }
    }
    pattern(x, y, "GSAS")
}

fn axis_key(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn check_axis(value: &str, format: &str) -> Result<(), Error> {
    let key = axis_key(value);
    if matches!(
        key.as_str(),
        "theta"
            | "omega"
            | "phi"
            | "chi"
            | "khi"
            | "psi"
            | "x"
            | "y"
            | "z"
            | "alpha"
            | "beta"
            | "q"
            | "d"
            | "tof"
            | "time"
            | "2thetarad"
    ) {
        return Err(invalid(format!(
            "{format}: axis {value:?} is not 2theta in degrees"
        )));
    }
    Ok(())
}

pub(crate) fn parse_ras(bytes: &[u8], scan: usize) -> Result<ParsedPattern, Error> {
    let text = decode(bytes)?;
    let mut index = 0;
    let mut active = false;
    let mut axis = String::new();
    let mut count = None;
    let mut x = Vec::new();
    let mut y = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        match line {
            "*RAS_HEADER_START" => {
                axis.clear();
                count = None;
            }
            "*RAS_INT_START" => {
                if active {
                    return Err(invalid("RAS: nested intensity blocks"));
                }
                active = true;
                if index == scan {
                    check_axis(&axis, "RAS")?;
                }
            }
            "*RAS_INT_END" => {
                if !active {
                    return Err(invalid(
                        "RAS: intensity block closes without an opening marker",
                    ));
                }
                if index == scan {
                    if count.is_some_and(|n| n != x.len()) {
                        return Err(invalid(
                            "RAS: declared point count disagrees with intensity block",
                        ));
                    }
                    return pattern(x, y, "RAS");
                }
                index += 1;
                active = false;
            }
            _ if active => {
                if index == scan {
                    let (a, b) = xy_row(line, "RAS")?;
                    x.push(a);
                    y.push(b);
                }
            }
            _ => {
                if let Some((key, value)) = line.split_once(char::is_whitespace) {
                    let value = value.trim().trim_matches('"');
                    if key == "*MEAS_SCAN_AXIS_X" {
                        axis = value.to_owned();
                    }
                    if key == "*MEAS_DATA_COUNT" {
                        count = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| invalid("RAS: invalid point count"))?,
                        );
                    }
                }
            }
        }
    }
    if active {
        return Err(invalid("RAS: file ends before *RAS_INT_END"));
    }
    Err(invalid(format!(
        "RAS: scan {scan} is out of range ({index} scans)"
    )))
}

pub(crate) fn parse_uxd(bytes: &[u8], scan: usize) -> Result<ParsedPattern, Error> {
    let text = decode(bytes)?;
    let mut header = HashMap::<String, String>::new();
    let mut opened = HashMap::new();
    let mut marker: Option<&str> = None;
    let mut index = 0;
    let mut x = Vec::new();
    let mut y = Vec::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(';'))
    {
        if matches!(line, "_COUNTS" | "_CPS" | "_2THETACOUNTS" | "_2THETACPS") {
            if marker.is_some() {
                if index == scan {
                    break;
                }
                index += 1;
            }
            marker = Some(line);
            if index == scan {
                opened = header.clone();
            }
        } else if let Some((key, value)) = line.strip_prefix('_').and_then(|s| s.split_once('=')) {
            header.insert(
                key.trim().to_ascii_uppercase(),
                value.trim().trim_matches(['\'', '"']).to_owned(),
            );
        } else if line.starts_with('_') {
            return Err(invalid(format!("UXD: unsupported block marker {line:?}")));
        } else {
            let marker = marker.ok_or_else(|| invalid("UXD: data precedes a counts block"))?;
            if index == scan {
                if marker.starts_with("_2THETA") {
                    let (a, b) = xy_row(line, "UXD")?;
                    x.push(a);
                    y.push(b);
                } else {
                    y.extend(fields(line).map(number).collect::<Result<Vec<_>, _>>()?);
                }
            }
        }
    }
    if marker.is_none() || index != scan {
        return Err(invalid(format!("UXD: scan {scan} is out of range")));
    }
    check_axis(opened.get("DRIVE").map(String::as_str).unwrap_or(""), "UXD")?;
    if x.is_empty() && !y.is_empty() {
        let start = number(
            opened
                .get("START")
                .ok_or_else(|| invalid("UXD: missing _START for counts-only data"))?,
        )?;
        let step = number(
            opened
                .get("STEPSIZE")
                .ok_or_else(|| invalid("UXD: missing _STEPSIZE for counts-only data"))?,
        )?;
        if step == 0.0 {
            return Err(invalid("UXD: _STEPSIZE must be nonzero"));
        }
        x = (0..y.len()).map(|i| start + i as f64 * step).collect();
    }
    pattern(x, y, "UXD")
}

pub(crate) fn parse_chi(bytes: &[u8]) -> Result<ParsedPattern, Error> {
    let text = decode(bytes)?;
    let mut lines = text.lines();
    lines
        .next()
        .ok_or_else(|| invalid("CHI: missing four-line header"))?;
    let label = lines
        .next()
        .ok_or_else(|| invalid("CHI: missing x axis label"))?
        .to_lowercase();
    let key = axis_key(&label);
    let is_angle =
        key.contains("2theta") || key.contains("2th") || key.contains("tth") || key.contains('θ');
    let words: Vec<&str> = label
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let is_other = words
        .iter()
        .any(|w| matches!(*w, "q" | "d" | "radial" | "pixel" | "channel"))
        || key.starts_with("qnm")
        || key.starts_with("qa")
        || key.contains("dspacing");
    if (!is_angle && is_other) || label.contains("radian") || label.contains("(rad)") {
        return Err(invalid(format!(
            "CHI: x axis {label:?} is not 2theta in degrees"
        )));
    }
    lines
        .next()
        .ok_or_else(|| invalid("CHI: missing intensity label"))?;
    let count_line = lines
        .next()
        .ok_or_else(|| invalid("CHI: missing point count"))?;
    let mut counts = count_line.split_whitespace();
    let count = counts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| invalid("CHI: invalid point count"))?;
    if let Some(datasets) = counts.next() {
        if datasets != "1" {
            return Err(invalid("CHI: only one dataset per profile is supported"));
        }
    }
    if counts.next().is_some() {
        return Err(invalid("CHI: invalid point-count header"));
    }
    let mut x = Vec::new();
    let mut y = Vec::new();
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let (a, b) = xy_row(line, "CHI")?;
        x.push(a);
        y.push(b);
    }
    if x.len() != count {
        return Err(invalid(format!(
            "CHI: header declares {count} points but {} were read",
            x.len()
        )));
    }
    pattern(x, y, "CHI")
}

#[derive(Debug)]
struct CifToken {
    value: String,
    quoted: bool,
}

impl CifToken {
    fn control(&self) -> bool {
        if self.quoted {
            return false;
        }
        let s = self.value.to_ascii_lowercase();
        s.starts_with('_')
            || s.starts_with("data_")
            || s.starts_with("save_")
            || matches!(s.as_str(), "loop_" | "stop_" | "global_")
    }
}

fn cif_tokens(text: &str) -> Result<Vec<CifToken>, Error> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'#' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b';' && (i == 0 || bytes[i - 1] == b'\n') {
            let start = i + 1;
            i = start;
            while i < bytes.len() && !(bytes[i] == b';' && bytes[i - 1] == b'\n') {
                i += 1;
            }
            if i == bytes.len() {
                return Err(invalid("pdCIF: unterminated semicolon text field"));
            }
            tokens.push(CifToken {
                value: text[start..i].to_owned(),
                quoted: true,
            });
            i += 1;
        } else if matches!(bytes[i], b'\'' | b'"') {
            let quote = bytes[i];
            i += 1;
            let start = i;
            while i < bytes.len()
                && !(bytes[i] == quote
                    && (i + 1 == bytes.len() || bytes[i + 1].is_ascii_whitespace()))
            {
                if bytes[i] == b'\n' {
                    return Err(invalid("pdCIF: unterminated quoted value"));
                }
                i += 1;
            }
            if i == bytes.len() {
                return Err(invalid("pdCIF: unterminated quoted value"));
            }
            tokens.push(CifToken {
                value: text[start..i].to_owned(),
                quoted: true,
            });
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(CifToken {
                value: text[start..i].to_owned(),
                quoted: false,
            });
        }
    }
    Ok(tokens)
}

fn cif_number(token: &str) -> Result<f64, Error> {
    if let Some(start) = token.find('(') {
        let end = token[start + 1..]
            .find(')')
            .map(|i| start + 1 + i)
            .ok_or_else(|| invalid("pdCIF: malformed numeric uncertainty"))?;
        if !token[start + 1..end].bytes().all(|c| c.is_ascii_digit()) || end == start + 1 {
            return Err(invalid("pdCIF: malformed numeric uncertainty"));
        }
        let plain = format!("{}{}", &token[..start], &token[end + 1..]);
        number(&plain)
    } else {
        number(token)
    }
}

#[derive(Default)]
struct CifBlock {
    name: String,
    columns: HashMap<String, Vec<String>>,
    scalars: HashMap<String, String>,
}

fn cif_column(block: &CifBlock, alternatives: &[&str]) -> Option<Vec<String>> {
    alternatives
        .iter()
        .find_map(|tag| block.columns.get(*tag).filter(|c| !c.is_empty()).cloned())
}

fn cif_2theta_axis(block: &CifBlock, point_count: usize) -> Result<Option<Vec<f64>>, Error> {
    // Corrected angles take precedence, whether explicit or evenly spaced.
    // Range increments are scalar spacing metadata, never angle columns.
    for (column, prefix) in [
        ("_pd_proc_2theta_corrected", "_pd_proc_2theta_range"),
        ("_pd_meas_2theta_scan", "_pd_meas_2theta_range"),
    ] {
        if let Some(col) = cif_column(block, &[column]) {
            return col
                .iter()
                .map(|s| cif_number(s))
                .collect::<Result<Vec<_>, _>>()
                .map(Some);
        }
        if let (Some(start), Some(step)) = (
            block.scalars.get(&format!("{prefix}_min")),
            block.scalars.get(&format!("{prefix}_inc")),
        ) {
            let start = cif_number(start)?;
            let step = cif_number(step)?;
            if step <= 0.0 {
                return Err(invalid("pdCIF: 2theta increment must be positive"));
            }
            let x: Vec<f64> = (0..point_count).map(|j| start + j as f64 * step).collect();
            if let Some(end) = block.scalars.get(&format!("{prefix}_max")) {
                let end = cif_number(end)?;
                if x.last()
                    .is_some_and(|last| (last - end).abs() > step.abs() * 1e-6 + 1e-8)
                {
                    return Err(invalid("pdCIF: angular range disagrees with point count"));
                }
            }
            return Ok(Some(x));
        }
    }
    Ok(None)
}

pub(crate) fn parse_pdcif(bytes: &[u8], selection: Option<&str>) -> Result<ParsedPattern, Error> {
    let text = decode(bytes)?;
    let tokens = cif_tokens(&text)?;
    let mut blocks = Vec::<CifBlock>::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let key = token.value.to_ascii_lowercase();
        if !token.quoted && key.starts_with("data_") {
            blocks.push(CifBlock {
                name: token.value[5..].to_owned(),
                ..CifBlock::default()
            });
            i += 1;
        } else if !token.quoted && key == "loop_" {
            let block = blocks
                .last_mut()
                .ok_or_else(|| invalid("pdCIF: loop precedes a data block"))?;
            i += 1;
            let mut tags = Vec::new();
            while i < tokens.len() && !tokens[i].quoted && tokens[i].value.starts_with('_') {
                tags.push(tokens[i].value.to_ascii_lowercase());
                i += 1;
            }
            if tags.is_empty() {
                return Err(invalid("pdCIF: loop has no column tags"));
            }
            let start = i;
            while i < tokens.len() && !tokens[i].control() {
                i += 1;
            }
            if (i - start) % tags.len() != 0 {
                return Err(invalid("pdCIF: incomplete loop row"));
            }
            for (offset, tag) in tags.iter().enumerate() {
                let col = (start + offset..i)
                    .step_by(tags.len())
                    .map(|j| tokens[j].value.clone())
                    .collect();
                if block.columns.insert(tag.clone(), col).is_some() {
                    return Err(invalid(format!("pdCIF: duplicate loop tag {tag}")));
                }
            }
        } else if !token.quoted && key.starts_with('_') {
            let block = blocks
                .last_mut()
                .ok_or_else(|| invalid("pdCIF: tag precedes a data block"))?;
            i += 1;
            if i == tokens.len() || tokens[i].control() {
                return Err(invalid(format!("pdCIF: missing value for {key}")));
            }
            block.scalars.insert(key, tokens[i].value.clone());
            i += 1;
        } else if !token.quoted && (key == "stop_" || key.starts_with("save_") || key == "global_")
        {
            i += 1;
        } else {
            return Err(invalid(format!(
                "pdCIF: unexpected token {:?}",
                token.value
            )));
        }
    }
    for block in blocks {
        if selection.is_some_and(|s| !block.name.contains(s)) {
            continue;
        }
        let Some(y) = cif_column(
            &block,
            &[
                "_pd_proc_intensity_total",
                "_pd_meas_intensity_total",
                "_pd_meas_counts_total",
                "_pd_proc_intensity_net",
            ],
        ) else {
            continue;
        };
        let y: Vec<f64> = y.iter().map(|s| cif_number(s)).collect::<Result<_, _>>()?;
        let Some(x) = cif_2theta_axis(&block, y.len())? else {
            continue;
        };
        if x.len() != y.len() {
            return Err(invalid(
                "pdCIF: 2theta and intensity columns have different lengths",
            ));
        }
        return pattern(x, y, "pdCIF");
    }
    Err(invalid(
        "pdCIF: no matching block with 2theta and intensity data",
    ))
}
