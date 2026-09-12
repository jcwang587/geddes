use geddes::{read_bytes, read_bytes_with_options, ReadOptions};

const X: &[f64] = &[10.0, 10.125, 10.25, 10.375, 10.5];
const Y: &[f64] = &[4.0, 81.0, 144.0, 9.0, 1.0];

fn assert_xy(bytes: &[u8], name: &str, x: &[f64], y: &[f64]) {
    let p = read_bytes(bytes, name).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert_eq!(p.x.len(), x.len(), "{name}: point count");
    assert_eq!(p.y, y, "{name}: intensity");
    for (actual, expected) in p.x.iter().zip(x) {
        assert!(
            (actual - expected).abs() < 1e-10,
            "{name}: {actual} != {expected}"
        );
    }
}

#[test]
fn same_profile_across_all_text_formats() {
    let fixtures: &[(&str, &[u8])] = &[
        ("profile.xy", include_bytes!("data/formats/text/profile.xy")),
        (
            "profile.csv",
            include_bytes!("data/formats/text/profile.csv"),
        ),
        (
            "profile.fxye",
            include_bytes!("data/formats/text/profile.fxye"),
        ),
        (
            "profile.gsas",
            include_bytes!("data/formats/text/profile.gsas"),
        ),
        (
            "profile_esd.gsas",
            include_bytes!("data/formats/text/profile_esd.gsas"),
        ),
        (
            "profile.ras",
            include_bytes!("data/formats/text/profile.ras"),
        ),
        (
            "profile.uxd",
            include_bytes!("data/formats/text/profile.uxd"),
        ),
        (
            "profile.chi",
            include_bytes!("data/formats/text/profile.chi"),
        ),
        (
            "profile.cif",
            include_bytes!("data/formats/text/profile.cif"),
        ),
    ];
    for (name, bytes) in fixtures {
        assert_xy(bytes, name, X, Y);
    }
}

#[test]
fn ascii_comments_bom_commas_and_unused_columns() {
    let bytes = "\u{feff}# header\n! comment\n; comment\n' comment\n/ comment\n2theta,intensity\n10,4,ignored\n10.125 81 garbage\n";
    assert_xy(bytes.as_bytes(), "pattern.xye", &X[..2], &Y[..2]);
}

#[test]
fn ascii_utf16_bom_little_and_big_endian() {
    let text = "# UTF-16 instrument export\n10 4\n10.125 81\n";
    for little in [true, false] {
        let mut bytes = if little {
            vec![0xff, 0xfe]
        } else {
            vec![0xfe, 0xff]
        };
        for ch in text.encode_utf16() {
            bytes.extend_from_slice(&if little {
                ch.to_le_bytes()
            } else {
                ch.to_be_bytes()
            });
        }
        assert_xy(&bytes, "pattern.xy", &X[..2], &Y[..2]);
    }
}

#[test]
fn ascii_common_extension_aliases() {
    for name in ["pattern.dat", "pattern.prn", "pattern.txt", "pattern.XYE"] {
        assert_xy(include_bytes!("data/formats/text/profile.xy"), name, X, Y);
    }
}

#[test]
fn ascii_empty_and_nonfinite_patterns_are_rejected() {
    for text in [
        "# no points\n",
        "10 NaN\n",
        "inf 10\n",
        "10 inf\n",
        "10 20\n10 30\n",
    ] {
        assert!(read_bytes(text, "pattern.xy").is_err(), "{text:?}");
    }
}

#[test]
fn descending_ascii_reverses_paired_values() {
    assert_xy(
        b"11 4\n10.5 9\n10 16\n",
        "descending.xy",
        &[10.0, 10.5, 11.0],
        &[16.0, 9.0, 4.0],
    );
}

#[test]
fn gsas_esd_fixed_fields_keep_fused_intensities_and_zero_esd_points() {
    let values = [
        (2_000.0, 44.7),
        (28_000.0, 167.3),
        (101_641.3, 0.0),
        (1_000_000.0, 1000.0),
        (42.0, 6.5),
    ];
    let fields: String = values
        .iter()
        .flat_map(|(y, e)| [format!("{y:8.1}"), format!("{e:8.1}")])
        .collect();
    // 1_000_000.0 is wider than the 8-character format; use an integer field
    // for this value to exercise a truly full-width fixed-format record.
    let fields = fields.replace("1000000.0", " 1000000");
    assert_eq!(fields.len(), 80);
    let text = format!("BANK 1 5 1 CONS 1000 12.5 0 0 ESD\n{fields}\n");
    assert_xy(
        text.as_bytes(),
        "profile.gsa",
        X,
        &[2_000.0, 28_000.0, 101_641.3, 1_000_000.0, 42.0],
    );
}

#[test]
fn gsas_esd_padding_and_short_final_record() {
    let record = [4.0, 2.0, 81.0, 0.0, 144.0, 12.0, 0.0, 0.0, 0.0, 0.0]
        .iter()
        .map(|v| format!("{v:8.1}"))
        .collect::<String>();
    let text = format!("BANK 1 3 1 CONST 1000 12.5 0 0 ESD\n{record}\n");
    assert_xy(text.as_bytes(), "profile.raw", &X[..3], &Y[..3]);
    let text = "BANK 1 2 1 CONST 1000 12.5 ESD\n     4.0     2.0    81.0     0.0\n";
    assert_xy(text.as_bytes(), "profile.gsas", &X[..2], &Y[..2]);
}

#[test]
fn gsas_preserves_explicit_irregular_fxye_axis() {
    let text = b"BANK 1 3 3 CONS 1000 1 0 0 FXYE\n1000 4 2\n1012.5 81 0\n1099 144 12\n";
    assert_xy(text, "profile.fxye", &[10.0, 10.125, 10.99], &Y[..3]);
}

#[test]
fn gsas_fxye_keeps_extra_endpoint_like_aps_11bm_exports() {
    // The real 11BM_NAC.fxye has 59,498 complete rows but declares 59,497.
    let text = b"BANK 1 2 2 CONS 1000 1 FXYE\n1000 4 2\n1012.5 81 0\n1025 144 12\n";
    assert_xy(text, "profile.fxye", &X[..3], &Y[..3]);
}

#[test]
fn gsas_bank_selection_is_zero_based() {
    let bytes = b"BANK 1 2 1 CONS 1000 100 STD\n4 9\nBANK 7 2 1 CONS 2000 50 STD\n25 36\n";
    let p = read_bytes_with_options(
        bytes,
        "pattern.gsas",
        &ReadOptions {
            scan: 1,
            block: None,
        },
    )
    .unwrap();
    assert_eq!(p.x, [20.0, 20.5]);
    assert_eq!(p.y, [25.0, 36.0]);
    assert!(read_bytes_with_options(
        bytes,
        "pattern.gsas",
        &ReadOptions {
            scan: 2,
            block: None
        }
    )
    .is_err());
}

#[test]
fn gsas_descending_constant_step_reverses_paired_values() {
    let text = b"BANK 1 3 1 CONS 1100 -50 STD\n4 9 16\n";
    assert_xy(text, "profile.gsas", &[10.0, 10.5, 11.0], &[16.0, 9.0, 4.0]);
}

#[test]
fn gsas_refuses_non_angle_binning_and_unsupported_record_types() {
    for bintype in [
        "COND", "CONQ", "RALF", "SLOG", "TIME_MAP", "EDS", "LOG6", "LPSD", "MYSTERY",
    ] {
        let text = format!("BANK 1 2 2 {bintype} 1000 20 0 0 FXYE\n1000 4 2\n1020 9 3\n");
        let err = read_bytes(text, "profile.gsas").unwrap_err().to_string();
        assert!(err.contains(bintype), "{err}");
    }
    for flag in ["ALT", "FXY", "UNKNOWN"] {
        let text = format!("BANK 1 2 2 CONS 1000 20 0 0 {flag}\n1000 4 2\n1020 9 3\n");
        let err = read_bytes(text, "profile.gsas").unwrap_err().to_string();
        assert!(err.contains(flag), "{err}");
    }
}

#[test]
fn gsas_refuses_truncated_and_corrupt_records() {
    for text in [
        "BANK 1 3 3 CONS 1000 20 STD\n4 9\n",
        "BANK 1 2 2 CONS 1000 20 FXYE\n1000 4 2\n1020 9\n",
        "BANK 1 2 2 CONS 1000 20 FXYE\n1000 4 2\n",
        "BANK 1 2 2 CONS 1000 20 ESD\n     4.0     2.0 BADBAD!     3.0\n",
        "BANK 1 1 1 CONS 1000 20 STD\n4 9\n",
        "BANK 1 0 0 CONS 1000 20 STD\n",
    ] {
        assert!(read_bytes(text, "profile.gsas").is_err(), "{text}");
    }
}

#[test]
fn ras_scan_selection_and_descending_reversal() {
    let bytes = include_bytes!("data/formats/text/profile.ras");
    let p = read_bytes_with_options(
        bytes,
        "profile.ras",
        &ReadOptions {
            scan: 1,
            block: None,
        },
    )
    .unwrap();
    assert_eq!(p.x, [20.0, 20.25, 20.5]);
    assert_eq!(p.y, [100.0, 64.0, 25.0]);
    assert!(read_bytes_with_options(
        bytes,
        "profile.ras",
        &ReadOptions {
            scan: 2,
            block: None
        }
    )
    .is_err());
}

#[test]
fn ras_rejects_rocking_curves_count_mismatch_and_unclosed_blocks() {
    let original = include_str!("data/formats/text/profile.ras");
    assert!(read_bytes(original.replace("TwoThetaTheta", "Omega"), "profile.ras").is_err());
    assert!(read_bytes(
        original.replace("*MEAS_DATA_COUNT \"5\"", "*MEAS_DATA_COUNT \"6\""),
        "profile.ras"
    )
    .is_err());
    assert!(read_bytes(
        b"*RAS_DATA_START\n*RAS_INT_START\n10 4\n11 9\n",
        "profile.ras"
    )
    .is_err());
    assert!(read_bytes(b"*RAS_DATA_START\n*RAS_INT_END\n", "profile.ras").is_err());
}

#[test]
fn ras_ignores_legacy_encoded_comment_text() {
    let mut bytes = b"*RAS_DATA_START\n*FILE_COMMENT \"".to_vec();
    bytes.extend_from_slice(&[0x83, 0x65, 0x83, 0x58, 0x83, 0x67]);
    bytes.extend_from_slice(b"\"\n*RAS_INT_START\n10 4\n11 9\n*RAS_INT_END\n");
    assert_xy(&bytes, "profile.ras", &[10.0, 11.0], &[4.0, 9.0]);
}

#[test]
fn uxd_selects_ranges_and_inherits_axis_parameters() {
    let bytes = include_bytes!("data/formats/text/profile.uxd");
    for scan in [1, 2] {
        let p = read_bytes_with_options(bytes, "profile.uxd", &ReadOptions { scan, block: None })
            .unwrap();
        assert_eq!(
            p.x,
            if scan == 1 {
                vec![20.0, 20.25, 20.5]
            } else {
                vec![20.0, 20.125, 20.25]
            }
        );
        assert_eq!(p.y, [100.0, 64.0, 25.0]);
    }
}

#[test]
fn uxd_all_four_markers_preserve_stored_intensity() {
    for marker in ["_COUNTS", "_CPS", "_2THETACOUNTS", "_2THETACPS"] {
        let data = if marker.starts_with("_2THETA") {
            "10 4\n11 9\n"
        } else {
            "4 9\n"
        };
        let text = format!("_FILEVERSION=2\n_DRIVE='2THETA'\n_START=10\n_STEPSIZE=1\n_STEPTIME=50\n{marker}\n{data}");
        assert_xy(text.as_bytes(), "profile.uxd", &[10.0, 11.0], &[4.0, 9.0]);
    }
}

#[test]
fn uxd_counts_only_descending_steps_reverse_paired_values() {
    for (marker, values, expected) in [
        ("_COUNTS", "9 25 49", [49.0, 25.0, 9.0]),
        ("_CPS", "0.9 2.5 4.9", [4.9, 2.5, 0.9]),
    ] {
        let text = format!(
            "_FILEVERSION=2\n_DRIVE='2THETA'\n_START=20.5\n_STEPSIZE=-0.25\n_STEPTIME=10\n{marker}\n{values}\n"
        );
        assert_xy(
            text.as_bytes(),
            "descending.uxd",
            &[20.0, 20.25, 20.5],
            &expected,
        );
    }
}

#[test]
fn uxd_selects_descending_range_with_its_own_start_and_step() {
    let bytes = b"_FILEVERSION=2\n_DRIVE='COUPLED'\n_START=10\n_STEPSIZE=0.5\n_COUNTS\n4 9\n_START=20.5\n_STEPSIZE=-0.25\n_CPS\n9.5 25.5 49.5\n_START=30\n_STEPSIZE=0.1\n_COUNTS\n1 2\n";
    assert_xy(bytes, "ranges.uxd", &[10.0, 10.5], &[4.0, 9.0]);
    let pattern = read_bytes_with_options(
        bytes,
        "ranges.uxd",
        &ReadOptions {
            scan: 1,
            block: None,
        },
    )
    .unwrap();
    assert_eq!(pattern.x, [20.0, 20.25, 20.5]);
    assert_eq!(pattern.y, [49.5, 25.5, 9.5]);
}

#[test]
fn uxd_counts_only_rejects_zero_and_nonfinite_steps() {
    for marker in ["_COUNTS", "_CPS"] {
        for step in ["0", "-0", "NaN", "inf", "-inf"] {
            let text = format!(
                "_FILEVERSION=2\n_DRIVE='2THETA'\n_START=20.5\n_STEPSIZE={step}\n{marker}\n9 25 49\n"
            );
            assert!(
                read_bytes(text, "invalid-step.uxd").is_err(),
                "{marker}: {step}"
            );
        }
    }
}

#[test]
fn uxd_rejects_non_diffraction_axes_and_missing_counts_axis() {
    for text in [
        "_FILEVERSION=2\n_DRIVE='THETA'\n_2THETACOUNTS\n10 4\n11 9\n",
        "_FILEVERSION=2\n_COUNTS\n4 9\n",
        "_FILEVERSION=2\n_START=10\n_STEPSIZE=0\n_COUNTS\n4 9\n",
        "_FILEVERSION=2\n_UNKNOWN\n10 4\n11 9\n",
        "_FILEVERSION=2\n_2THETACOUNTS\n10\n11\n",
    ] {
        assert!(read_bytes(text, "profile.uxd").is_err(), "{text}");
    }
}

#[test]
fn chi_count_header_never_becomes_a_point() {
    assert_xy(
        b"title\n2theta\ncounts\n2 1\n10 4\n11 9\n",
        "pattern.chi",
        &[10.0, 11.0],
        &[4.0, 9.0],
    );
}

#[test]
fn chi_rejects_other_axes_and_invalid_headers() {
    for label in [
        "q (A^-1)",
        "Q_nm^-1",
        "d-spacing (A)",
        "pixel radius",
        "2theta (radians)",
    ] {
        let text = format!("title\n{label}\ncounts\n2 1\n10 4\n11 9\n");
        assert!(read_bytes(text, "pattern.chi").is_err(), "{label}");
    }
    for text in [
        "title\n2theta\ncounts\n3\n10 4\n11 9\n",
        "title\n2theta\ncounts\n2\n10 4\n11\n",
        "title\n2theta\ncounts\n2 2\n10 4\n11 9\n",
    ] {
        assert!(read_bytes(text, "pattern.chi").is_err(), "{text}");
    }
}

#[test]
fn pdcif_selects_matching_named_profile_block() {
    let bytes = include_bytes!("data/formats/text/profile.cif");
    let p = read_bytes_with_options(
        bytes,
        "profile.cif",
        &ReadOptions {
            scan: 0,
            block: Some("_other".into()),
        },
    )
    .unwrap();
    assert_eq!(p.x, [20.0, 20.25, 20.5]);
    assert_eq!(p.y, [100.0, 64.0, 25.0]);
    assert!(read_bytes_with_options(
        bytes,
        "profile.cif",
        &ReadOptions {
            scan: 0,
            block: Some("missing".into())
        }
    )
    .is_err());
}

#[test]
fn pdcif_profile_tag_priority_and_exponent_uncertainties() {
    let text = b"data_profile\nloop_\n_PD_PROC_2THETA_CORRECTED\n_pd_meas_2theta_scan\n_pd_proc_intensity_total\n_pd_meas_counts_total\n1.00(2)e1 10.1 4(2) 40\n1.10(2)e1 11.1 9(3) 90\n";
    assert_xy(text, "pattern.cif", &[10.0, 11.0], &[4.0, 9.0]);
}

#[test]
fn pdcif_scalar_angular_range_builds_axis_for_counts_loop() {
    let text = b"data_profile\n_pd_meas_2theta_range_min 10\n_pd_meas_2theta_range_max 11\n_pd_meas_2theta_range_inc .5\nloop_\n_pd_meas_intensity_total\n4 9 16\n";
    assert_xy(text, "pattern.cif", &[10.0, 10.5, 11.0], &[4.0, 9.0, 16.0]);
}

#[test]
fn pdcif_rejects_incomplete_loops_missing_values_and_structure_only_cif() {
    for text in [
        "data_profile\nloop_\n_pd_meas_2theta_scan\n_pd_meas_counts_total\n10 4 11\n",
        "data_profile\nloop_\n_pd_meas_2theta_scan\n_pd_meas_counts_total\n10 4\n11 ?\n",
        "data_profile\nloop_\n_pd_meas_2theta_scan\n_pd_meas_counts_total\n10 4\n11 .\n",
        "data_structure\n_cell_length_a 5.4\n",
        "data_structure\n_chemical_name_common\n;never closes\n",
        "data_structure\n_chemical_name_common 'never closes\n",
    ] {
        assert!(read_bytes(text, "pattern.cif").is_err(), "{text}");
    }
}
