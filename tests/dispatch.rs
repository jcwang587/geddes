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
            scan: 1,
            block: None
        }
    )
    .is_err());
}
