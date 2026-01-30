use geddes::{read, read_bytes};
use std::fs::read as fs_read;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn test_01_read_gsas_raw() {
    let path = PathBuf::from("tests/data/gsas_raw/sample.raw");
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
    let path = PathBuf::from("tests/data/gsas_raw/sample.raw");
    let start = Instant::now();
    let bytes = fs_read(&path).expect("Failed to read file bytes");
    println!("IO time (read bytes) for GSAS raw: {:?}", start.elapsed());
    let pattern = read_bytes(&bytes, "sample.raw").expect("Failed to load raw from bytes");
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
