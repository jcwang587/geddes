use geddes::{read_bytes, read_bytes_with_options, ReadOptions};
use std::io::{Cursor, Write};

fn archive(members: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, text) in members {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn selected(bytes: &[u8], name: &str, scan: usize) -> geddes::Pattern {
    read_bytes_with_options(bytes, name, &ReadOptions { scan, block: None }).unwrap()
}

fn xrdml_scan(axis: &str, positions: &str, values: &str) -> String {
    format!("<scan scanAxis=\"{axis}\"><dataPoints><positions axis=\"2Theta\" unit=\"deg\">{positions}</positions>{values}</dataPoints></scan>")
}

fn xrdml(scans: &str) -> String {
    format!("<xrdMeasurements><xrdMeasurement>{scans}</xrdMeasurement></xrdMeasurements>")
}

const RANGE: &str = "<startPosition>10</startPosition><endPosition>11</endPosition>";

#[test]
fn bundled_xml_containers_match_every_known_point() {
    let expected_x = [10., 10.125, 10.25, 10.375, 10.5];
    let expected_y = [4., 81., 144., 9., 1.];
    for (bytes, name) in [
        (
            include_bytes!("data/formats/xml/synthetic.xrdml").as_slice(),
            "synthetic.xrdml",
        ),
        (
            include_bytes!("data/formats/xml/synthetic.rasx").as_slice(),
            "synthetic.rasx",
        ),
        (
            include_bytes!("data/formats/xml/synthetic.brml").as_slice(),
            "synthetic.brml",
        ),
    ] {
        let pattern = read_bytes(bytes, name).unwrap();
        assert_eq!(pattern.x, expected_x, "{name}");
        assert_eq!(pattern.y, expected_y, "{name}");
    }
}

#[test]
fn xrdml_corrects_counts_but_preserves_reported_intensities_and_cps() {
    let factors = "<beamAttenuationFactors>100 10 1</beamAttenuationFactors><commonCountingTime>2</commonCountingTime>";
    for (tag, unit, expected) in [
        ("counts", "counts", vec![200., 30., 4.]),
        ("intensities", "counts", vec![2., 3., 4.]),
        ("intensities", "cps", vec![2., 3., 4.]),
    ] {
        let values = format!("{factors}<{tag} unit=\"{unit}\">2 3 4</{tag}>");
        let source = xrdml(&xrdml_scan("Omega2Theta", RANGE, &values));
        let p = read_bytes(source, "pattern.xrdml").unwrap();
        assert_eq!(p.x, [10., 10.5, 11.]);
        assert_eq!(p.y, expected);
    }
}

#[test]
fn xrdml_supports_position_layouts_namespaces_entities_cdata_and_bom() {
    let source = "\u{feff}<x:xrdMeasurements xmlns:x=\"urn:test\"><x:xrdMeasurement><x:scan scanAxis=\"Gonio\"><x:dataPoints><x:positions axis=\"2Theta\" unit=\"deg\"><x:listPositions>10 10.3 11</x:listPositions></x:positions><x:counts><![CDATA[2 3]]>&#32;4</x:counts></x:dataPoints></x:scan></x:xrdMeasurement></x:xrdMeasurements>";
    let p = read_bytes(source, "namespaced.xrdml").unwrap();
    assert_eq!(p.x, [10., 10.3, 11.]);
    assert_eq!(p.y, [2., 3., 4.]);
    let single = xrdml(&xrdml_scan(
        "2Theta",
        "<commonPosition>&#49;0</commonPosition>",
        "<counts>7</counts>",
    ));
    let p = read_bytes(single, "single.xrdml").unwrap();
    assert_eq!(p.x, [10.]);
    assert_eq!(p.y, [7.]);
    let reversed = xrdml(&xrdml_scan(
        "Gonio",
        "<listPositions>11 10.5 10</listPositions>",
        "<counts>2 3 4</counts>",
    ));
    let p = read_bytes(reversed, "reverse.xrdml").unwrap();
    assert_eq!(p.x, [10., 10.5, 11.]);
    assert_eq!(p.y, [4., 3., 2.]);
}

#[test]
fn xrdml_accepts_escaped_ampersands_in_attributes() {
    for description in [
        "Research &amp; Development",
        "Research &#38; Development",
        "Research &#x26; Development",
        "Literal &amp;custom; reference",
    ] {
        let source = xrdml(&xrdml_scan("&#71;onio", RANGE, "<counts>1 2</counts>")).replace(
            "<xrdMeasurements>",
            &format!("<xrdMeasurements description=\"{description}\">"),
        );
        let p = read_bytes(source, "attributes.xrdml").unwrap();
        assert_eq!(p.x, [10., 11.]);
        assert_eq!(p.y, [1., 2.]);
    }
}

#[test]
fn xrdml_selects_scans_across_measurements_without_merging() {
    let first = xrdml_scan("Gonio", RANGE, "<counts>1 2</counts>");
    let second = xrdml_scan("Gonio", RANGE, "<intensities>30 40</intensities>");
    let source = format!("<xrdMeasurements><xrdMeasurement>{first}</xrdMeasurement><xrdMeasurement>{second}</xrdMeasurement></xrdMeasurements>");
    assert_eq!(read_bytes(&source, "multi.xrdml").unwrap().y, [1., 2.]);
    assert_eq!(selected(source.as_bytes(), "multi.xrdml", 1).y, [30., 40.]);
    assert!(read_bytes_with_options(
        source,
        "multi.xrdml",
        &ReadOptions {
            scan: 2,
            block: None
        }
    )
    .is_err());
}

#[test]
fn xrdml_rejects_invalid_lengths_axes_units_and_truncated_xml() {
    for scan in [
        xrdml_scan(
            "Gonio",
            "<listPositions>10 11 12</listPositions>",
            "<counts>1 2</counts>",
        ),
        xrdml_scan(
            "Gonio",
            RANGE,
            "<counts>1 2</counts><beamAttenuationFactors>1</beamAttenuationFactors>",
        ),
        xrdml_scan(
            "Gonio",
            RANGE,
            "<counts>1 2</counts><beamAttenuationFactors>1 0</beamAttenuationFactors>",
        ),
        xrdml_scan("Omega", RANGE, "<counts>1 2</counts>"),
        xrdml_scan(
            "Gonio",
            "<commonPosition>10</commonPosition>",
            "<counts>1 2</counts>",
        ),
        xrdml_scan("Gonio", RANGE, "<counts>1 nope</counts>"),
        xrdml_scan("Gonio", RANGE, "<counts>1 NaN</counts>"),
        xrdml_scan("Gonio", RANGE, "<counts/>"),
    ] {
        assert!(read_bytes(xrdml(&scan), "bad.xrdml").is_err(), "{scan}");
    }
    let valid = xrdml(&xrdml_scan("Gonio", RANGE, "<counts>1 2</counts>"));
    assert!(read_bytes(
        valid.replace("unit=\"deg\"", "unit=\"rad\""),
        "radians.xrdml"
    )
    .is_err());
    assert!(read_bytes(&valid[..valid.len() - 4], "truncated.xrdml").is_err());
}

#[test]
fn rasx_follows_manifest_order_and_reads_only_the_selected_profile() {
    let root = "<Root><Data9><ContentHashList Name=\"Profile9.txt\"/><ContentHashList Name=\"Conditions9.xml\"/></Data9><Data1><ContentHashList Name=\"Profile1.txt\"/></Data1></Root>";
    let conditions = "<MeasurementConditions><Other><AxisName>Omega</AxisName></Other><ScanInformation><AxisName>TwoThetaOmega</AxisName></ScanInformation></MeasurementConditions>";
    let bytes = archive(&[
        ("Data1/Profile1.txt", "20 9\n20.1 10\n20.2 11\n"),
        ("Data9/Conditions9.xml", conditions),
        ("Data9/Profile9.txt", "\u{feff}10 4 100\n10.125 81 1\n"),
        ("root.xml", root),
    ]);
    let first = read_bytes(&bytes, "any.dat").unwrap();
    assert_eq!(first.x, [10., 10.125]);
    assert_eq!(first.y, [4., 81.]); // Rigaku's third column is preserved by ignoring it.
    let second = selected(&bytes, "any.dat", 1);
    assert_eq!(second.x, [20., 20.1, 20.2]);
    assert_eq!(second.y, [9., 10., 11.]);
    assert!(read_bytes_with_options(
        bytes,
        "many.rasx",
        &ReadOptions {
            scan: 2,
            block: None
        }
    )
    .is_err());
}

#[test]
fn rasx_decodes_manifest_attributes_exactly_once() {
    for (encoded_name, member_name) in [
        ("Profile&amp;amp;.txt", "Data0/Profile&amp;.txt"),
        ("Profile&amp;#49;.txt", "Data0/Profile&#49;.txt"),
        ("Profile&amp;.txt", "Data0/Profile&.txt"),
    ] {
        let root =
            format!("<Root><Data0><ContentHashList Name=\"{encoded_name}\"/></Data0></Root>");
        let bytes = archive(&[("root.xml", &root), (member_name, "10 4\n11 5\n")]);
        let p = read_bytes(bytes, "attributes.rasx").unwrap();
        assert_eq!(p.x, [10., 11.]);
        assert_eq!(p.y, [4., 5.]);
    }
}

#[test]
fn rasx_rejects_missing_members_bad_rows_and_non_diffraction_scans() {
    let root = "<Root><Data0><ContentHashList Name=\"Profile0.txt\"/><ContentHashList Name=\"Conditions.xml\"/></Data0></Root>";
    let conditions = "<MeasurementConditions><ScanInformation><AxisName>Omega</AxisName></ScanInformation></MeasurementConditions>";
    for members in [
        vec![("root.xml", root)],
        vec![("Data0/Profile0.txt", "10 4\n11 5\n")],
        vec![
            ("root.xml", root),
            ("Data0/Profile0.txt", "10 4\n11 5\n"),
            ("Data0/Conditions.xml", conditions),
        ],
        vec![
            (
                "root.xml",
                "<Root><Data0><ContentHashList Name=\"Profile0.txt\"/></Data0></Root>",
            ),
            ("Data0/Profile0.txt", "10 4\n11\n"),
        ],
    ] {
        assert!(read_bytes(archive(&members), "bad.rasx").is_err());
    }
}

fn brml_route(flag: &str, axis: &str, width: usize, values: &[i32]) -> String {
    let rows: String = values
        .iter()
        .enumerate()
        .map(|(i, y)| {
            let angle = 10. + i as f64 * 0.125;
            format!("<Datum>1,{y},0,{},{} </Datum>", angle / 2., angle)
        })
        .collect();
    format!("<DataRoute RouteFlag=\"{flag}\"><DataViews><RawDataView xsi:type=\"VaryingRawDataView\" Start=\"3\" Length=\"2\"><FieldDefinitions AxisId=\"Theta\"/>{axis}</RawDataView><RawDataView xsi:type=\"RecordedRawDataView\" Start=\"1\" Length=\"{width}\"/></DataViews>{rows}</DataRoute>")
}

fn raw_data(routes: &str) -> String {
    format!("<RawData xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">{routes}</RawData>")
}

const TWO_THETA: &str = "<FieldDefinitions AxisId=\"TwoTheta\"/>";

fn brml_single(routes: &str) -> Vec<u8> {
    archive(&[
        ("Experiment0/DataContainer.xml", "<DataContainer><RawDataReferenceList><string>Experiment0/RawData0.xml</string></RawDataReferenceList></DataContainer>"),
        ("Experiment0/RawData0.xml", &raw_data(routes)),
    ])
}

#[test]
fn brml_reads_dynamic_columns_and_chooses_measured_route() {
    let measured = brml_route(
        "Measured",
        "<FieldDefinitions AxisId=\"\" FieldName=\" TwoTheta \"/>",
        1,
        &[4, 81, 144],
    );
    let processed = brml_route("Processed", TWO_THETA, 1, &[999, 999, 999]);
    let bytes = brml_single(&(processed.clone() + &measured));
    let p = read_bytes(bytes, "channels.brml").unwrap();
    assert_eq!(p.x, [10., 10.125, 10.25]);
    assert_eq!(p.y, [4., 81., 144.]);
    assert_eq!(
        read_bytes(brml_single(&processed), "only.brml").unwrap().y,
        [999., 999., 999.]
    );
    assert!(read_bytes(
        brml_single(&(measured.clone() + &measured)),
        "ambiguous.brml"
    )
    .is_err());
}

#[test]
fn brml_scan_selection_uses_manifest_order_instead_of_zip_order() {
    let first = raw_data(&brml_route("Measured", TWO_THETA, 1, &[4, 81]));
    let second = raw_data(&brml_route("Measured", TWO_THETA, 1, &[144, 9, 1]));
    let bytes = archive(&[
        ("Experiment0/RawData1.xml", &second),
        ("Experiment0/RawData9.xml", &first),
        ("Experiment0/DataContainer.xml", "<DataContainer><RawDataReferenceList><string>Experiment0/RawData9.xml</string><string>Experiment0/RawData1.xml</string></RawDataReferenceList></DataContainer>"),
    ]);
    assert_eq!(read_bytes(&bytes, "generic.bin").unwrap().y, [4., 81.]);
    assert_eq!(selected(&bytes, "generic.bin", 1).y, [144., 9., 1.]);
    assert!(read_bytes_with_options(
        bytes,
        "many.brml",
        &ReadOptions {
            scan: 2,
            block: None
        }
    )
    .is_err());
}

#[test]
fn brml_rejects_detector_frames_non_two_theta_and_missing_data() {
    for route in [
        brml_route("Measured", TWO_THETA, 2, &[4, 81]),
        brml_route(
            "Measured",
            "<FieldDefinitions AxisId=\"Chi\"/>",
            1,
            &[4, 81],
        ),
        brml_route("Measured", TWO_THETA, 1, &[]),
        brml_route("Measured", TWO_THETA, 1, &[4, 81]).replace("Start=\"1\"", "Start=\"99\""),
        brml_route("Measured", TWO_THETA, 1, &[4, 81])
            .replace("<Datum>1,4,0,5,10 </Datum>", "<Datum>1,4</Datum>"),
    ] {
        assert!(read_bytes(brml_single(&route), "invalid.brml").is_err());
    }
    let missing = archive(&[("DataContainer.xml", "<DataContainer><RawDataReferenceList><string>Missing.xml</string></RawDataReferenceList></DataContainer>")]);
    assert!(read_bytes(missing, "missing.brml").is_err());
}
