use geddes::{read_bytes, read_bytes_with_options, ReadOptions};

#[test]
fn content_wins_over_misleading_extensions() {
    for (bytes, names) in [
        (
            include_bytes!("data/formats/text/profile.uxd").as_slice(),
            vec!["wrong.raw", "wrong.cif"],
        ),
        (
            include_bytes!("data/formats/text/profile.gsas").as_slice(),
            vec!["wrong.cif", "wrong.uxd"],
        ),
        (
            include_bytes!("data/formats/text/profile.chi").as_slice(),
            vec!["wrong.raw", "wrong.xy"],
        ),
    ] {
        for name in names {
            let pattern = read_bytes(bytes, name).unwrap();
            assert_eq!(pattern.x, [10., 10.125, 10.25, 10.375, 10.5]);
            assert_eq!(pattern.y, [4., 81., 144., 9., 1.]);
        }
    }
    let raw_title = b"RAW diffraction data\nBANK 1 2 1 CONS 1000 100 0 0 STD\n1 2\n";
    assert_eq!(read_bytes(raw_title, "a.raw").unwrap().y, [1., 2.]);
}

#[test]
fn a_chi_shaped_ascii_header_does_not_claim_a_false_point_count() {
    let bytes = b"title\n2theta\nintensity\n5\n10 1\n11 2\n";
    assert_eq!(read_bytes(bytes, "a.xy").unwrap().x, [10., 11.]);
    assert!(read_bytes(bytes, "a.chi").is_err());
}

#[test]
fn an_integer_xy_row_is_not_a_chi_point_count() {
    let bytes = b"sample\n2theta\nintensity\n3 12\n4 24\n5 48\n6 96\n";
    let pattern = read_bytes(bytes, "a.xy").unwrap();
    assert_eq!(pattern.x, [3., 4., 5., 6.]);
    assert_eq!(pattern.y, [12., 24., 48., 96.]);
    assert!(read_bytes(bytes, "a.chi").is_err());

    let chi = b"sample\n2theta\nintensity\n3 1\n4 24\n5 48\n6 96\n";
    let pattern = read_bytes(chi, "a.dat").unwrap();
    assert_eq!(pattern.x, [4., 5., 6.]);
    assert_eq!(pattern.y, [24., 48., 96.]);
}

#[test]
fn xrdml_markers_in_ascii_headers_do_not_select_the_xml_reader() {
    for header in [
        "# Exported from <xrdMeasurements>",
        "Source: vendor:xrdMeasurements",
        "<!-- <xrdMeasurements> -->",
        "<note><xrdMeasurements/></note>",
        "<xrdMeasurementsHistory/>",
    ] {
        let bytes = format!("{header}\n10 12\n11 24\n");
        let pattern = read_bytes(bytes.as_bytes(), "a.xy").unwrap();
        assert_eq!(pattern.x, [10., 11.]);
        assert_eq!(pattern.y, [12., 24.]);
    }
}

#[test]
fn xrdml_root_detection_preserves_prologs_namespaces_and_parse_errors() {
    let document = "<?xml version=\"1.0\"?>\n<!-- measured profile -->\n<?instrument export?>\n\
        <x:xrdMeasurements xmlns:x=\"urn:xrdml\"><x:xrdMeasurement><x:scan><x:dataPoints>\
        <x:positions axis=\"2Theta\" unit=\"deg\"><x:startPosition>10</x:startPosition>\
        <x:endPosition>11</x:endPosition></x:positions>\
        <x:intensities unit=\"counts\">12 24</x:intensities>\
        </x:dataPoints></x:scan></x:xrdMeasurement></x:xrdMeasurements>";
    let pattern = read_bytes(document.as_bytes(), "a.xy").unwrap();
    assert_eq!(pattern.x, [10., 11.]);
    assert_eq!(pattern.y, [12., 24.]);

    let invalid = b"<xrdMeasurements>\n10 12\n11 24\n";
    assert!(read_bytes(invalid, "a.xy").is_err());
}

#[test]
fn peak_lists_and_unavailable_scans_are_rejected() {
    let dif = b"2theta intensity h k l\n10 100 1 0 0\n20 10 1 1 0\n30 4 1 1 1\n";
    assert!(read_bytes(dif, "a.dif")
        .unwrap_err()
        .to_string()
        .contains("peak lists"));
    let xy = b"10 1\n11 2\n";
    assert_eq!(read_bytes(xy, "renamed.dif").unwrap().x, [10., 11.]);
    assert!(read_bytes_with_options(
        xy,
        "a.xy",
        &ReadOptions {
            index: 1,
            block: None
        }
    )
    .is_err());
}
