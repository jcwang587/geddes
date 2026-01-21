use geddes::{load_file, load_from_reader};
use std::path::PathBuf;
use std::io::Cursor;
use std::fs::read;

#[test]
fn test_load_xy() {
    let path = PathBuf::from("tests/data/xy/sample.xy");
    let pattern = load_file(&path).expect("Failed to load xy file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from xy", pattern.x.len());
}

#[test]
fn test_load_rasx() {
    let path = PathBuf::from("tests/data/rasx/sample.rasx");
    let pattern = load_file(&path).expect("Failed to load rasx file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from rasx", pattern.x.len());
}

#[test]
fn test_load_raw() {
    let path = PathBuf::from("tests/data/raw/sample.raw");
    let pattern = load_file(&path).expect("Failed to load raw file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from raw", pattern.x.len());
}

#[test]
fn test_load_csv() {
    let path = PathBuf::from("tests/data/csv/sample.csv");
    let pattern = load_file(&path).expect("Failed to load csv file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    assert!(pattern.e.as_ref().map(|v| v.len() == pattern.x.len()).unwrap_or(true));
    println!("Loaded {} points from csv", pattern.x.len());
}

#[test]
fn test_load_from_bytes_xy() {
    let path = PathBuf::from("tests/data/xy/sample.xy");
    let bytes = read(&path).expect("Failed to read file bytes");
    let cursor = Cursor::new(bytes);
    
    let pattern = load_from_reader(cursor, "sample.xy").expect("Failed to load xy from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_load_from_bytes_rasx() {
    let path = PathBuf::from("tests/data/rasx/sample.rasx");
    let bytes = read(&path).expect("Failed to read file bytes");
    let cursor = Cursor::new(bytes);
    
    let pattern = load_from_reader(cursor, "sample.rasx").expect("Failed to load rasx from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
}

#[test]
fn test_load_from_bytes_csv() {
    let path = PathBuf::from("tests/data/csv/sample.csv");
    let bytes = read(&path).expect("Failed to read file bytes");
    let cursor = Cursor::new(bytes);
    
    let pattern = load_from_reader(cursor, "sample.csv").expect("Failed to load csv from bytes");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    assert!(pattern.e.as_ref().map(|v| v.len() == pattern.x.len()).unwrap_or(true));
}
