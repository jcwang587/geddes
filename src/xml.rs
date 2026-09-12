//! XML and ZIP pattern containers. Only the selected profile is interpreted.
use crate::parser::{decode, ParsedPattern};
use crate::Error;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

const MAX_MEMBER_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Default, Debug)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    text: String,
    children: Vec<Element>,
}

impl Element {
    fn attr(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
    fn child(&self, name: &str) -> Option<&Self> {
        self.children.iter().find(|c| c.name == name)
    }
    fn descendants<'a>(&'a self, name: &str, out: &mut Vec<&'a Self>) {
        if self.name == name {
            out.push(self);
        }
        for child in &self.children {
            child.descendants(name, out);
        }
    }
    fn all(&self, name: &str) -> Vec<&Self> {
        let mut out = Vec::new();
        self.descendants(name, &mut out);
        out
    }
}

fn error(msg: impl Into<String>) -> Error {
    Error::Parse(msg.into())
}

fn unescape(value: &str) -> Result<String, Error> {
    quick_xml::escape::unescape(value)
        .map(|v| v.into_owned())
        .map_err(|e| error(format!("XML entity: {e}")))
}

/// Create a node with local names and normalized, unescaped attribute values.
fn element(start: &BytesStart<'_>) -> Result<Element, Error> {
    let mut node = Element {
        name: start.local_name().as_ref().to_owned(),
        ..Element::default()
    };
    for attr in start.attributes() {
        let attr = attr.map_err(|e| error(format!("XML attribute: {e}")))?;
        let key = attr
            .key
            .as_ref()
            .rsplit(':')
            .next()
            .unwrap_or("")
            .to_owned();
        let value = attr
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|e| error(format!("XML attribute: {e}")))?;
        node.attrs.push((key, value.into_owned()));
    }
    Ok(node)
}

/// Decode vendor XML into a single element tree.
///
/// Resolve references once, preserve CDATA literally, and reject DTDs.
fn document(bytes: &[u8]) -> Result<Element, Error> {
    let text = decode(bytes)?;
    let mut reader = Reader::from_str(&text);
    let mut stack = vec![Element::default()];
    loop {
        match reader
            .read_event()
            .map_err(|e| error(format!("XML parse: {e}")))?
        {
            Event::Start(start) => {
                if stack.len() > 128 {
                    return Err(error("XML nesting exceeds 128 elements"));
                }
                stack.push(element(&start)?);
            }
            Event::Empty(start) => stack.last_mut().unwrap().children.push(element(&start)?),
            Event::Text(text) => stack.last_mut().unwrap().text.push_str(&text),
            Event::CData(text) => stack.last_mut().unwrap().text.push_str(&text),
            Event::GeneralRef(reference) => {
                let entity = format!("&{};", reference.as_ref());
                stack.last_mut().unwrap().text.push_str(&unescape(&entity)?);
            }
            Event::End(_) => {
                if stack.len() == 1 {
                    return Err(error("unexpected XML closing element"));
                }
                let node = stack.pop().unwrap();
                stack.last_mut().unwrap().children.push(node);
            }
            Event::DocType(_) => {
                return Err(error("XML document type declarations are not supported"))
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if stack.len() != 1 {
        return Err(error("truncated XML document"));
    }
    let mut wrapper = stack.pop().unwrap();
    if wrapper.children.len() != 1 || !wrapper.text.trim().is_empty() {
        return Err(error("XML must contain exactly one root element"));
    }
    Ok(wrapper.children.remove(0))
}

fn numbers(text: &str) -> Result<Vec<f64>, Error> {
    text.split_whitespace().map(number).collect()
}
fn number(value: &str) -> Result<f64, Error> {
    let v = value
        .trim()
        .parse::<f64>()
        .map_err(|_| error(format!("invalid numeric value {value:?}")))?;
    if !v.is_finite() {
        return Err(error("non-finite numeric value"));
    }
    Ok(v)
}
fn index(value: Option<&str>) -> Result<usize, Error> {
    value
        .ok_or_else(|| error("missing column Start/Length"))?
        .parse()
        .map_err(|_| error("invalid column Start/Length"))
}
fn select<T>(values: &[T], scan: usize) -> Result<&T, Error> {
    values.get(scan).ok_or_else(|| {
        error(format!(
            "scan {scan} is out of range ({} scans)",
            values.len()
        ))
    })
}
fn axis_key(axis: &str) -> String {
    axis.to_lowercase().replace([' ', '-', '_'], "")
}
fn check_rigaku_axis(axis: &str) -> Result<(), Error> {
    if !matches!(
        axis_key(axis).as_str(),
        "twotheta"
            | "twothetatheta"
            | "twothetaomega"
            | "2theta"
            | "2thetaw"
            | "2thetatheta"
            | "2thetaomega"
    ) {
        return Err(error(format!(
            "scan axis {axis:?} is not a supported 2theta scan"
        )));
    }
    Ok(())
}

pub(crate) fn parse_xrdml(bytes: &[u8], scan: usize) -> Result<ParsedPattern, Error> {
    let root = document(bytes)?;
    if root.name != "xrdMeasurements" {
        return Err(error("not an XRDML measurement"));
    }
    let scans: Vec<_> = root
        .children
        .iter()
        .filter(|m| m.name == "xrdMeasurement")
        .flat_map(|m| m.children.iter().filter(|s| s.name == "scan"))
        .collect();
    let scan = select(&scans, scan)?;
    if let Some(axis) = scan.attr("scanAxis") {
        if !matches!(
            axis_key(axis).as_str(),
            "gonio"
                | "2theta"
                | "2thetaomega"
                | "omega2theta"
                | "omegatwotheta"
                | "twotheta"
                | "omegatheta"
                | "2thetatheta"
        ) {
            return Err(error(format!("XRDML scan axis {axis:?} is not 2theta")));
        }
    }
    let points = scan
        .child("dataPoints")
        .ok_or_else(|| error("XRDML scan has no dataPoints"))?;
    let intensity = points
        .child("intensities")
        .or_else(|| points.child("counts"))
        .ok_or_else(|| error("XRDML missing intensities/counts"))?;
    let mut y = numbers(&intensity.text)?;
    if y.is_empty() {
        return Err(error("XRDML has no intensity points"));
    }
    let positions = points
        .children
        .iter()
        .find(|p| p.name == "positions" && p.attr("axis") == Some("2Theta"))
        .ok_or_else(|| error("XRDML missing 2Theta positions"))?;
    if let Some(unit) = positions.attr("unit") {
        if !matches!(
            unit.to_lowercase().as_str(),
            "deg" | "degree" | "degrees" | "°"
        ) {
            return Err(error(format!("XRDML 2Theta unit {unit:?} is not degrees")));
        }
    }
    let x = if let Some(list) = positions.child("listPositions") {
        let x = numbers(&list.text)?;
        if x.len() != y.len() {
            return Err(error("XRDML position/intensity lengths differ"));
        }
        x
    } else if let (Some(start), Some(end)) = (
        positions.child("startPosition"),
        positions.child("endPosition"),
    ) {
        let start = number(&start.text)?;
        let end = number(&end.text)?;
        if y.len() == 1 {
            vec![start]
        } else {
            (0..y.len())
                .map(|i| start + (end - start) * i as f64 / (y.len() - 1) as f64)
                .collect()
        }
    } else if y.len() == 1 {
        vec![number(
            &positions
                .child("commonPosition")
                .ok_or_else(|| error("XRDML missing stepped 2Theta positions"))?
                .text,
        )?]
    } else {
        return Err(error(
            "XRDML has a fixed or missing 2Theta axis, not a powder profile",
        ));
    };
    if let Some(factors) = points.child("beamAttenuationFactors") {
        let factors = numbers(&factors.text)?;
        if factors.len() != y.len() || factors.iter().any(|v| *v <= 0.) {
            return Err(error(
                "XRDML attenuation factors must be positive and match intensity length",
            ));
        }
        // <counts> stores detector counts; <intensities> is already corrected.
        if intensity.name == "counts" {
            for (y, factor) in y.iter_mut().zip(factors) {
                *y *= factor;
            }
        }
    }
    Ok(ParsedPattern { x, y })
}

fn member<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Result<Vec<u8>, Error> {
    let file = archive.by_name(name).map_err(|e| match e {
        zip::result::ZipError::FileNotFound => Error::FileNotFoundInArchive(name.to_owned()),
        other => Error::Zip(other),
    })?;
    let mut bytes = Vec::new();
    file.take(MAX_MEMBER_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_MEMBER_BYTES {
        return Err(error("archive pattern member exceeds 256 MiB"));
    }
    Ok(bytes)
}

pub(crate) fn parse_rasx(bytes: &[u8], scan: usize) -> Result<ParsedPattern, Error> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let root = document(&member(&mut archive, "root.xml")?)?;
    let mut groups = Vec::new();
    for group in &root.children {
        if !group
            .name
            .strip_prefix("Data")
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
        {
            continue;
        }
        let names: Vec<_> = group
            .children
            .iter()
            .filter(|e| e.name == "ContentHashList")
            .filter_map(|e| e.attr("Name"))
            .collect();
        if let Some(profile) = names.iter().find(|n| n.to_lowercase().ends_with(".txt")) {
            let conditions = names.iter().find(|n| {
                n.to_lowercase().contains("conditions") && n.to_lowercase().ends_with(".xml")
            });
            groups.push((
                format!("{}/{}", group.name, profile),
                conditions.map(|n| format!("{}/{n}", group.name)),
            ));
        }
    }
    let (profile, conditions) = select(&groups, scan)?;
    if let Some(conditions) = conditions {
        let root = document(&member(&mut archive, conditions)?)?;
        let scans = root.all("ScanInformation");
        let axes = scans.first().copied().unwrap_or(&root).all("AxisName");
        if let Some(axis) = axes.first() {
            check_rigaku_axis(axis.text.trim())?;
        }
    }
    let raw = member(&mut archive, profile)?;
    let text = decode(&raw)?;
    let mut x = Vec::new();
    let mut y = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let mut fields = line.split_whitespace();
        x.push(number(
            fields.next().ok_or_else(|| error("RASX missing 2theta"))?,
        )?);
        y.push(number(
            fields
                .next()
                .ok_or_else(|| error("RASX missing intensity"))?,
        )?);
    }
    Ok(ParsedPattern { x, y })
}

pub(crate) fn parse_brml(bytes: &[u8], scan: usize) -> Result<ParsedPattern, Error> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let manifest = archive
        .file_names()
        .find(|n| n.ends_with("DataContainer.xml"))
        .map(str::to_owned)
        .ok_or_else(|| error("BRML missing DataContainer.xml"))?;
    let root = document(&member(&mut archive, &manifest)?)?;
    let members: Vec<String> = root
        .all("RawDataReferenceList")
        .iter()
        .flat_map(|e| &e.children)
        .map(|e| e.text.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    let raw = document(&member(&mut archive, select(&members, scan)?)?)?;
    let routes = raw.all("DataRoute");
    let measured: Vec<_> = routes
        .iter()
        .copied()
        .filter(|r| r.attr("RouteFlag") == Some("Measured"))
        .collect();
    let route = if measured.len() == 1 {
        measured[0]
    } else if routes.len() == 1 {
        routes[0]
    } else {
        return Err(error("BRML requires one unambiguous measured data route"));
    };
    let mut x_column = None;
    let mut y_column = None;
    for view in route.all("RawDataView") {
        let start = index(view.attr("Start"))?;
        let length = index(view.attr("Length"))?;
        match view
            .attr("type")
            .unwrap_or("")
            .rsplit(':')
            .next()
            .unwrap_or("")
        {
            "VaryingRawDataView" => {
                let fields = view.all("FieldDefinitions");
                if fields.len() > length {
                    return Err(error("BRML fields exceed declared view length"));
                }
                for (offset, field) in fields.iter().enumerate() {
                    let axis = field
                        .attr("AxisId")
                        .filter(|a| !a.trim().is_empty())
                        .or_else(|| field.attr("FieldName"))
                        .unwrap_or("")
                        .trim();
                    if axis.eq_ignore_ascii_case("TwoTheta") && x_column.is_none() {
                        x_column = Some(
                            start
                                .checked_add(offset)
                                .ok_or_else(|| error("BRML column overflow"))?,
                        );
                    }
                }
            }
            "RecordedRawDataView" => {
                if length != 1 {
                    return Err(error(
                        "BRML detector frames are not integrated XRD profiles",
                    ));
                }
                if y_column.is_none() {
                    y_column = Some(start);
                }
            }
            _ => {}
        }
    }
    let xi = x_column.ok_or_else(|| error("BRML has no varying TwoTheta column"))?;
    let yi = y_column.ok_or_else(|| error("BRML has no recorded intensity column"))?;
    let rows = route.all("Datum");
    let mut x = Vec::with_capacity(rows.len());
    let mut y = Vec::with_capacity(rows.len());
    for row in rows {
        let mut px = None;
        let mut py = None;
        for (i, value) in row.text.split(',').enumerate() {
            if i == xi {
                px = Some(number(value)?);
            }
            if i == yi {
                py = Some(number(value)?);
            }
        }
        x.push(px.ok_or_else(|| error("BRML datum missing TwoTheta column"))?);
        y.push(py.ok_or_else(|| error("BRML datum missing intensity column"))?);
    }
    // Bruker has already applied its absorption factor to the stored intensity.
    Ok(ParsedPattern { x, y })
}
