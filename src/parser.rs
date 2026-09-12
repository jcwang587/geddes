use crate::{bruker, text, xml, Error, ReadOptions};
use quick_xml::{events::Event, Reader};
use std::borrow::Cow;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct ParsedPattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

/// Decode vendor text, including Windows UTF-16 exports identified by a BOM.
pub(crate) fn decode(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
    if bytes.starts_with(b"\xff\xfe") || bytes.starts_with(b"\xfe\xff") {
        if !bytes.len().is_multiple_of(2) {
            return Err(Error::Parse("truncated UTF-16 text".into()));
        }
        let little = bytes[0] == 0xff;
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|v| {
                if little {
                    u16::from_le_bytes([v[0], v[1]])
                } else {
                    u16::from_be_bytes([v[0], v[1]])
                }
            })
            .collect();
        return String::from_utf16(&words)
            .map(Cow::Owned)
            .map_err(|_| Error::Parse("invalid UTF-16 text".into()));
    }
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    // Legacy headers may contain non-UTF8 names; numeric data are ASCII.
    Ok(String::from_utf8_lossy(bytes))
}

fn one_pattern(options: &ReadOptions) -> Result<(), Error> {
    if options.index != 0 {
        return Err(Error::Parse(
            "this format has one pattern; index must be 0".into(),
        ));
    }
    Ok(())
}

fn has_xrdml_root(content: &str) -> bool {
    if !content.trim_start().starts_with('<') {
        return false;
    }
    let mut reader = Reader::from_str(content);
    loop {
        match reader.read_event() {
            Ok(Event::Start(root) | Event::Empty(root)) => {
                return root.local_name().as_ref() == "xrdMeasurements";
            }
            Ok(Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_)) => {}
            Ok(Event::Text(text)) if text.as_bytes().iter().all(u8::is_ascii_whitespace) => {}
            _ => return false,
        }
    }
}

pub(crate) fn parse(
    bytes: &[u8],
    filename: &str,
    options: &ReadOptions,
) -> Result<ParsedPattern, Error> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if bytes.starts_with(b"RAW1.01")
        || bytes.starts_with(b"RAW4.00")
        || bytes.starts_with(b"RAW2")
        || bytes.starts_with(b"RAW ") && bytes[..bytes.len().min(256)].contains(&0)
    {
        return bruker::parse_bruker_raw(bytes, options.index);
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        let archive = zip::ZipArchive::new(Cursor::new(bytes))?;
        if archive
            .file_names()
            .any(|n| n.ends_with("DataContainer.xml"))
        {
            return xml::parse_brml(bytes, options.index);
        }
        if archive
            .file_names()
            .any(|n| n == "root.xml" || (n.contains("Profile") && n.ends_with(".txt")))
        {
            return xml::parse_rasx(bytes, options.index);
        }
        return Err(Error::UnknownFormat);
    }
    let content = decode(bytes)?;
    let mut end = content.len().min(65536);
    while !content.is_char_boundary(end) {
        end -= 1;
    }
    let head = &content[..end];
    if head.contains('\0') {
        return Err(Error::UnknownFormat);
    }
    // All strong content recognizers run before extension fallbacks.
    if has_xrdml_root(&content) {
        return xml::parse_xrdml(bytes, options.index);
    }
    if head.lines().any(|l| {
        l.trim_start().starts_with("*RAS_DATA_START")
            || l.trim_start().starts_with("*RAS_HEADER_START")
    }) {
        return text::parse_ras(bytes, options.index);
    }
    if head.lines().any(|l| l.trim_start().starts_with("data_")) && head.contains("_pd_") {
        one_pattern(options)?;
        return text::parse_pdcif(bytes, options.block.as_deref());
    }
    if head
        .lines()
        .any(|l| l.split_whitespace().next() == Some("BANK"))
    {
        return text::parse_gsas(bytes, options.index);
    }
    if head
        .lines()
        .any(|l| l.trim_start().starts_with("_DRIVE=") || l.trim_start().starts_with("_DRIVE ="))
    {
        return text::parse_uxd(bytes, options.index);
    }
    let lines: Vec<&str> = head.lines().take(5).collect();
    let chi_header = lines.len() >= 5
        && lines[..3].iter().all(|l| {
            let s = l.trim();
            !s.is_empty()
                && !s.starts_with(['#', '!', ';'])
                && s.split_whitespace()
                    .take(2)
                    .any(|v| v.parse::<f64>().is_err())
        })
        && {
            let fields: Vec<_> = lines[3].split_whitespace().collect();
            !fields.is_empty()
                && fields.len() <= 2
                && (fields.len() == 1 || fields[1] == "1")
                && fields.iter().all(|v| v.parse::<usize>().is_ok())
                && fields[0].parse::<usize>().ok()
                    == Some(
                        content
                            .lines()
                            .skip(4)
                            .filter(|l| !l.trim().is_empty())
                            .count(),
                    )
        };
    if chi_header {
        one_pattern(options)?;
        return text::parse_chi(bytes);
    }
    match ext.as_str() {
        "xrdml" => return xml::parse_xrdml(bytes, options.index),
        "ras" => return text::parse_ras(bytes, options.index),
        "uxd" => return text::parse_uxd(bytes, options.index),
        "gsas" | "gsa" | "fxye" | "gda" | "xra" | "raw" => {
            return text::parse_gsas(bytes, options.index)
        }
        "cif" => {
            one_pattern(options)?;
            return text::parse_pdcif(bytes, options.block.as_deref());
        }
        "chi" => {
            one_pattern(options)?;
            return text::parse_chi(bytes);
        }
        "rasx" | "brml" => {
            return Err(Error::Parse(format!("{ext} is not a readable ZIP archive")))
        }
        "dif" => {
            let reflections = head
                .lines()
                .filter(|line| {
                    let p: Vec<_> = line.split_whitespace().collect();
                    (p.len() == 5 || p.len() == 6)
                        && p[..2].iter().all(|v| v.parse::<f64>().is_ok())
                        && p[p.len() - 3..].iter().all(|v| v.parse::<i32>().is_ok())
                })
                .count();
            if reflections >= 3
                || head.lines().any(|l| {
                    l.to_lowercase()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .windows(3)
                        .any(|w| w == ["h", "k", "l"])
                })
            {
                return Err(Error::Parse(
                    "DIF reflection peak lists are not measured XRD profiles".into(),
                ));
            }
        }
        _ => {}
    }
    one_pattern(options)?;
    text::parse_xy(bytes)
}
