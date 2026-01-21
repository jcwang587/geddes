# Geddes

A Rust library to parse XRD pattern files. Supports:
- `.xy` / `.xye` (ASCII, space separated)
- `.rasx` (Rigaku, Zip containing Profile text)
- `.raw` (GSAS format, text based)
- `.csv` (comma separated values)

## Usage

```rust
use geddes::load_file;

fn main() {
    let pattern = load_file("tests/data/xy/sample.xy").unwrap();
    println!("Loaded {} points", pattern.x.len());
}
```

## License

MIT
