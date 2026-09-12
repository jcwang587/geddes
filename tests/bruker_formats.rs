use geddes::{read_bytes, read_bytes_with_options, ReadOptions};

fn put_u32(buf: &mut [u8], at: usize, value: u32) {
    buf[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_f64(buf: &mut [u8], at: usize, value: f64) {
    buf[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

// Literal field offsets are deliberately independent of the reader. Each
// record contains an intensity and non-intensity payload to test its stride.
fn raw4_scan(start: f64, step: f64, y: &[f32], stride: usize, drive: &str) -> Vec<u8> {
    let mut out = vec![0; 160 + 92];
    out[32..46].copy_from_slice(b"Locked Coupled");
    put_f64(&mut out, 72, start);
    put_f64(&mut out, 80, step);
    put_u32(&mut out, 88, y.len() as u32);
    put_u32(&mut out, 136, stride as u32);
    put_u32(&mut out, 140, 92);
    put_u32(&mut out, 160, 50);
    put_u32(&mut out, 164, 92);
    put_u32(&mut out, 168, 2);
    out[172..172 + drive.len()].copy_from_slice(drive.as_bytes());
    put_f64(&mut out, 216, start);
    for value in y {
        out.extend_from_slice(&value.to_le_bytes());
        out.extend(vec![0x7f; stride - 4]);
    }
    out
}

fn raw4(scans: &[Vec<u8>]) -> Vec<u8> {
    let mut out = vec![0; 61];
    out[..8].copy_from_slice(b"RAW4.00\0");
    for scan in scans {
        out.extend_from_slice(scan);
    }
    out
}

fn raw3_scan(start: f64, step: f64, y: &[f32], measured: Option<&[f64]>) -> Vec<u8> {
    // 304-byte header + one 40-byte optional record; two varying parameters
    // exercise both the measured 2theta column and an additional ignored one.
    let mut out = vec![0; 344];
    put_u32(&mut out, 0, 304);
    put_u32(&mut out, 4, y.len() as u32);
    put_f64(&mut out, 16, start);
    put_f64(&mut out, 176, step);
    put_u32(&mut out, 248, if measured.is_some() { 3 } else { 0 });
    put_u32(&mut out, 252, if measured.is_some() { 20 } else { 4 });
    put_u32(&mut out, 256, 40);
    put_u32(&mut out, 304, 10);
    put_u32(&mut out, 308, 40);
    for (i, value) in y.iter().enumerate() {
        out.extend_from_slice(&value.to_le_bytes());
        if let Some(x) = measured {
            out.extend_from_slice(&x[i].to_le_bytes());
            out.extend_from_slice(&(x[i] / 2.).to_le_bytes());
        }
    }
    out
}

fn raw3(scans: &[Vec<u8>]) -> Vec<u8> {
    let mut out = vec![0; 712];
    out[..8].copy_from_slice(b"RAW1.01\0");
    put_u32(&mut out, 12, scans.len() as u32);
    for scan in scans {
        out.extend_from_slice(scan);
    }
    out
}

#[test]
fn bundled_binary_patterns_match_the_complete_known_pattern() {
    let expected = read_bytes(
        include_bytes!("data/formats/bruker/expected.xy"),
        "expected.xy",
    )
    .unwrap();
    for fixture in [
        include_bytes!("data/formats/bruker/synthetic_raw3.raw").as_slice(),
        include_bytes!("data/formats/bruker/synthetic_raw4_4byte.raw").as_slice(),
        include_bytes!("data/formats/bruker/synthetic_raw4_8byte.raw").as_slice(),
    ] {
        let result = read_bytes(fixture, "pattern.raw").unwrap();
        assert_eq!(result.y, expected.y);
        assert_eq!(result.x.len(), expected.x.len());
        for (&actual, &expected) in result.x.iter().zip(&expected.x) {
            assert!((actual - expected).abs() < 1e-12);
        }
    }
}

#[test]
fn raw4_reads_declared_strides_and_selects_scans() {
    let mut second = raw4_scan(30., 0.5, &[123., 2.5, -7.], 12, "2Theta");
    put_u32(&mut second, 0, 160); // alternate range marker
    let bytes = raw4(&[raw4_scan(10., 1., &[1., 2.], 4, "2Theta"), second]);
    assert_eq!(read_bytes(&bytes, "test.raw").unwrap().y, [1., 2.]);
    let p = read_bytes_with_options(
        &bytes,
        "test.raw",
        &ReadOptions {
            index: 1,
            block: None,
        },
    )
    .unwrap();
    assert_eq!(p.x, [30., 30.5, 31.]);
    assert_eq!(p.y, [123., 2.5, -7.]);
    assert!(read_bytes_with_options(
        &bytes,
        "test.raw",
        &ReadOptions {
            index: 2,
            block: None
        }
    )
    .is_err());
}

#[test]
fn raw3_measured_positions_extra_records_and_multiple_scans() {
    let bytes = raw3(&[
        raw3_scan(10., 0.5, &[7., 8.], None),
        raw3_scan(30., 0.5, &[101., 99., 1.], Some(&[30.01, 30.49, 31.02])),
    ]);
    assert_eq!(read_bytes(&bytes, "test.raw").unwrap().x, [10., 10.5]);
    let p = read_bytes_with_options(
        &bytes,
        "test.raw",
        &ReadOptions {
            index: 1,
            block: None,
        },
    )
    .unwrap();
    assert_eq!(p.x, [30.01, 30.49, 31.02]);
    assert_eq!(p.y, [101., 99., 1.]);
    let mut padded = bytes.clone();
    padded.extend([0; 3280]);
    assert!(read_bytes(&padded, "test.raw").is_ok());
    padded.push(1);
    assert!(read_bytes(&padded, "test.raw").is_err());
}

#[test]
fn descending_binary_scans_keep_intensity_paired_with_angle() {
    let bytes = raw4(&[raw4_scan(30., -0.5, &[2., 8., 11.], 8, "2Theta")]);
    let p = read_bytes(&bytes, "test.raw").unwrap();
    assert_eq!(p.x, [29., 29.5, 30.]);
    assert_eq!(p.y, [11., 8., 2.]);
}

#[test]
fn rejects_unsupported_versions_and_non_diffraction_axes() {
    for bytes in [b"RAW \0\0\0\0".as_slice(), b"RAW2\0\0\0\0".as_slice()] {
        let error = read_bytes(bytes, "old.raw").unwrap_err().to_string();
        assert!(error.contains("unsupported"), "{error}");
    }
    let mut scan3 = raw3_scan(5., 1., &[1., 2.], None);
    put_u32(&mut scan3, 196, 3); // rocking curve
    for bytes in [
        raw4(&[raw4_scan(5., 1., &[1., 2.], 4, "Theta")]),
        raw4(&[raw4_scan(5., 1., &[1., 2.], 4, "Unknown")]),
        raw3(&[scan3]),
    ] {
        let error = read_bytes(bytes, "other.raw").unwrap_err().to_string();
        assert!(error.contains("2theta"), "{error}");
    }
}

#[test]
fn rejects_truncation_false_strides_and_drive_header_overruns() {
    let good = raw4(&[raw4_scan(10., 1., &[1., 2., 3.], 8, "2Theta")]);
    for cut in [7, 60, 62, 120, good.len() - 1] {
        assert!(read_bytes(&good[..cut], "truncated.raw").is_err());
    }
    for (offset, value) in [(61 + 136, 3), (61 + 140, 93), (61 + 164, 96), (61 + 164, 0)] {
        let mut broken = good.clone();
        put_u32(&mut broken, offset, value);
        assert!(
            read_bytes(&broken, "broken.raw").is_err(),
            "offset={offset}"
        );
    }
    let mut broken3 = raw3(&[raw3_scan(10., 1., &[1., 2.], None)]);
    put_u32(&mut broken3, 712 + 252, 12); // no varying parameters declared
    assert!(read_bytes(&broken3, "broken.raw").is_err());
    let invalid = raw4(&[raw4_scan(10., 1., &[f32::NAN, 2.], 4, "2Theta")]);
    assert!(read_bytes(&invalid, "nonfinite.raw").is_err());
}

#[test]
fn existing_raw4_files_decode_all_stored_intensity_records() {
    // Exact data offsets are independently verified by walking the real
    // files' headers; EVA's scrambled intensities establish byte fidelity.
    for (bytes, offset, points, stride, start, step) in [
        (
            include_bytes!("data/bruker_raw/bruker4_v5converter.raw").as_slice(),
            8117,
            4059,
            4,
            37.0001,
            0.020454544980000003,
        ),
        (
            include_bytes!("data/bruker_raw/bruker4_diffrac_eva.raw").as_slice(),
            1311,
            7134,
            8,
            10.,
            0.010520299896597862,
        ),
    ] {
        let p = read_bytes(bytes, "actual.raw").unwrap();
        assert_eq!(p.x.len(), points);
        for i in 0..points {
            let at = offset + i * stride;
            let expected_y = f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as f64;
            assert_eq!(p.y[i], expected_y);
            assert!((p.x[i] - (start + i as f64 * step)).abs() < 1e-12);
        }
    }
}
