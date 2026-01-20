use geddes::load_file;
use std::path::PathBuf;

#[test]
fn test_load_xy() {
    let path = PathBuf::from("test/data/xy/Y2O3_vesta.xy");
    let pattern = load_file(&path).expect("Failed to load xy file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from xy", pattern.x.len());
}

#[test]
fn test_load_rasx() {
    let path = PathBuf::from("test/data/rasx/LNO_2025-0916.rasx");
    let pattern = load_file(&path).expect("Failed to load rasx file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from rasx", pattern.x.len());
}

#[test]
fn test_load_raw() {
    let path = PathBuf::from("test/data/raw/CoO25C.raw");
    let pattern = load_file(&path).expect("Failed to load raw file");
    assert!(pattern.x.len() > 0);
    assert_eq!(pattern.x.len(), pattern.y.len());
    println!("Loaded {} points from raw", pattern.x.len());
}
