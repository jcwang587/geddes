# Geddes

A Rust library to parse XRD pattern files. Supports:
- `.xy` / `.xye` (ASCII, space separated)
- `.rasx` (Rigaku, Zip containing Profile text)
- `.raw` (GSAS format, text based)

## Usage

```rust
use geddes::load_file;
use std::path::Path;

fn main() {
    let pattern = load_file(Path::new("test/data/xy/Y2O3_vesta.xy")).unwrap();
    println!("Loaded {} points", pattern.x.len());
    
    // To JSON
    let json = serde_json::to_string(&pattern).unwrap();
}
```

## Features

- **Fast Parsing**: Optimized for performance.
- **Multiple Formats**: Handles common XRD formats.
- **Serde Support**: Structs implement `Serialize` and `Deserialize` for easy JSON conversion.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
geddes = "0.1.0"
```

## For Electron / JavaScript

This crate is designed to be easily compiled to WebAssembly (WASM) or used with N-API for Electron applications.

## License

MIT
