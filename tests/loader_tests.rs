use geddes::{read, read_bytes};
use std::fs::read as fs_read;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn test_01_read_gsas_raw() {
    let path = PathBuf::from("tests/data/gsas_raw/gsas.raw");
    let start = Instant::now();
    let pattern = read(&path).expect("Failed to load raw file");
    println!("IO time for GSAS raw: {:?}", start.elapsed());
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from GSAS raw", pattern.x.len());
}

#[test]
fn test_02_read_bruker_raw() {
    let path = PathBuf::from("tests/data/bruker_raw/bruker.raw");
    let start = Instant::now();
    let pattern = read(&path).expect("Failed to load Bruker raw file");
    println!("IO time for Bruker raw: {:?}", start.elapsed());
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from Bruker raw", pattern.x.len());
}

#[test]
fn test_03_read_rasx() {
    let path = PathBuf::from("tests/data/rasx/sample.rasx");
    let start = Instant::now();
    let pattern = read(&path).expect("Failed to load rasx file");
    println!("IO time for rasx: {:?}", start.elapsed());
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from rasx", pattern.x.len());
}

#[test]
fn test_04_read_xrdml() {
    let path = PathBuf::from("tests/data/xrdml/sample.xrdml");
    let start = Instant::now();
    let pattern = read(&path).expect("Failed to load xrdml file");
    println!("IO time for xrdml: {:?}", start.elapsed());
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from xrdml", pattern.x.len());
}

#[test]
fn test_05_read_xy() {
    let path = PathBuf::from("tests/data/xy/sample.xy");
    let start = Instant::now();
    let pattern = read(&path).expect("Failed to load xy file");
    println!("IO time for xy: {:?}", start.elapsed());
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from xy", pattern.x.len());
}

#[test]
fn test_06_read_csv() {
    let path = PathBuf::from("tests/data/csv/sample.csv");
    let start = Instant::now();
    let pattern = read(&path).expect("Failed to load csv file");
    println!("IO time for csv: {:?}", start.elapsed());
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    assert!(pattern
        .e
        .as_ref()
        .map(|v| v.len() == pattern.x.len())
        .unwrap_or(true));
    println!("Loaded {} points from csv", pattern.x.len());
}

#[test]
fn test_07_read_bytes_gsas_raw() {
    let path = PathBuf::from("tests/data/gsas_raw/gsas.raw");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read file bytes");
    println!("IO time (read bytes) for GSAS raw: {:?}", start.elapsed());
    let pattern = read_bytes(&bytes, "gsas.raw").expect("Failed to load raw from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_08_read_bytes_bruker_raw() {
    let path = PathBuf::from("tests/data/bruker_raw/bruker.raw");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read Bruker raw bytes");
    println!(
        "IO time (read bytes) for Bruker raw: {:?}",
        start.elapsed()
    );
    let pattern =
        read_bytes(&bytes, "bruker.raw").expect("Failed to load Bruker raw from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_09_read_bytes_rasx() {
    let path = PathBuf::from("tests/data/rasx/sample.rasx");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read file bytes");
    println!("IO time (read bytes) for rasx: {:?}", start.elapsed());
    let pattern = read_bytes(&bytes, "sample.rasx").expect("Failed to load rasx from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_10_read_bytes_xrdml() {
    let path = PathBuf::from("tests/data/xrdml/sample.xrdml");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read file bytes");
    println!("IO time (read bytes) for xrdml: {:?}", start.elapsed());
    let pattern =
        read_bytes(&bytes, "sample.xrdml").expect("Failed to load xrdml from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_11_read_bytes_xy() {
    let path = PathBuf::from("tests/data/xy/sample.xy");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read file bytes");
    println!("IO time (read bytes) for xy: {:?}", start.elapsed());
    let pattern = read_bytes(&bytes, "sample.xy").expect("Failed to load xy from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_12_read_bytes_csv() {
    let path = PathBuf::from("tests/data/csv/sample.csv");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read file bytes");
    println!("IO time (read bytes) for csv: {:?}", start.elapsed());
    let pattern = read_bytes(&bytes, "sample.csv").expect("Failed to load csv from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    assert!(pattern
        .e
        .as_ref()
        .map(|v| v.len() == pattern.x.len())
        .unwrap_or(true));
}

#[test]
fn test_13_bruker_raw_axis_span_is_physical() {
    let path = PathBuf::from("tests/data/bruker_raw/bruker.raw");
    let pattern = read(&path).expect("Failed to load Bruker raw file");
    assert_eq!(pattern.x.len(), pattern.y.len());
    assert!(pattern.x.len() > 10);

    let x_start = *pattern.x.first().expect("Missing x start");
    let x_end = *pattern.x.last().expect("Missing x end");
    let x_span = x_end - x_start;

    // The old heuristic could return near-zero step sizes.
    assert!(x_span > 1.0, "Bruker x span is unexpectedly tiny: {x_span}");
    assert!(
        x_span < 360.0,
        "Bruker x span is unexpectedly large for a scan: {x_span}"
    );
}

#[test]
fn test_14_bruker_raw_scrambled_does_not_include_marker_words() {
    let path = PathBuf::from("tests/data/bruker_raw/TwoTheta_scan_scrambled.raw");
    let pattern = read(&path).expect("Failed to load scrambled Bruker raw file");
    assert_eq!(pattern.x.len(), pattern.y.len());
    assert!(pattern.x.len() > 100);

    let marker = f32::from_bits(1) as f64;
    let marker_like = pattern
        .y
        .iter()
        .filter(|&&v| (v - marker).abs() < 1.0e-50)
        .count();
    assert!(
        marker_like * 10 < pattern.y.len(),
        "Too many marker-like words leaked into intensity stream: {marker_like}/{}",
        pattern.y.len()
    );
}
