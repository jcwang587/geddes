use geddes::{read_bytes_with_options, read_with_options, ReadOptions};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Manifest {
    fixtures: Vec<Fixture>,
}
#[derive(Deserialize)]
struct Fixture {
    id: String,
    path: String,
    reference: String,
    points: usize,
    scan: usize,
    block: Option<String>,
    rtol: f64,
    atol: f64,
}

#[test]
fn every_fixture_matches_every_reference_point_from_path_and_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/formats");
    let manifest: Manifest =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    for fixture in manifest.fixtures {
        let source = root.join(&fixture.path);
        let reference = std::fs::read_to_string(root.join(&fixture.reference)).unwrap();
        let rows: Vec<(f64, f64)> = reference
            .lines()
            .skip(1)
            .map(|line| {
                let (x, y) = line.split_once(',').unwrap();
                (x.parse().unwrap(), y.parse().unwrap())
            })
            .collect();
        assert_eq!(
            rows.len(),
            fixture.points,
            "{} reference length",
            fixture.id
        );
        let options = ReadOptions {
            scan: fixture.scan,
            block: fixture.block,
        };
        let path_pattern = read_with_options(&source, &options)
            .unwrap_or_else(|e| panic!("{} path: {e}", fixture.id));
        let bytes = std::fs::read(&source).unwrap();
        let byte_pattern = read_bytes_with_options(&bytes, source.to_str().unwrap(), &options)
            .unwrap_or_else(|e| panic!("{} bytes: {e}", fixture.id));
        for (mode, pattern) in [("path", path_pattern), ("bytes", byte_pattern)] {
            assert_eq!(pattern.x.len(), rows.len(), "{} {mode} length", fixture.id);
            assert_eq!(pattern.y.len(), rows.len());
            for (i, ((x, y), (rx, ry))) in pattern.x.iter().zip(&pattern.y).zip(&rows).enumerate() {
                assert!(
                    (x - rx).abs() <= fixture.atol + fixture.rtol * rx.abs(),
                    "{} {mode} x[{i}]: {x} != {rx}",
                    fixture.id
                );
                assert!(
                    (y - ry).abs() <= fixture.atol + fixture.rtol * ry.abs(),
                    "{} {mode} y[{i}]: {y} != {ry}",
                    fixture.id
                );
            }
        }
    }
}
