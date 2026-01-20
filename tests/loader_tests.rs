use geddes::load_file;
use std::path::PathBuf;

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
