# Geddes

[![Crates.io](https://img.shields.io/crates/v/geddes)](https://crates.io/crates/geddes)
[![PyPI](https://img.shields.io/pypi/v/geddes)](https://pypi.org/project/geddes/)


A Rust XRD pattern parser with Python bindings. Supports:
- `.raw` (GSAS format, text based; Bruker format, binary)
- `.rasx` (Rigaku, Zip containing Profile text)
- `.xrdml` (Panalytical XML-based format)
- `.xy` / `.xye` (ASCII, space-separated values)
- `.csv` (comma-separated values)

## Usage

Load from a file path:

```rust
use geddes::load_pattern;

fn main() {
    let pattern = load_pattern("tests/data/xy/sample.xy").unwrap();
    println!("{} {}", pattern.x.len(), pattern.y.len());
}
```

Load from in-memory bytes (filename is used to infer the format):

```rust
use std::fs;
use std::io::Cursor;

use geddes::load_pattern_from_reader;

fn main() {
    let data = fs::read("tests/data/xy/sample.xy").unwrap();
    let cursor = Cursor::new(data);
    let pattern = load_pattern_from_reader(cursor, "sample.xy").unwrap();
    println!("{} {}", pattern.x.len(), pattern.y.len());
}
```

## Python Usage

This crate ships Python bindings via `pyo3`/`maturin`.

Load from a file path:

```python
import geddes

pattern = geddes.load_pattern("tests/data/xy/sample.xy")
print(len(pattern.x), len(pattern.y))
```

Load from in-memory bytes (filename is used to infer the format):

```python
import geddes

with open("tests/data/xy/sample.xy", "rb") as f:
    data = f.read()

pattern = geddes.load_pattern_from_bytes(data, "sample.xy")
print(len(pattern.x), len(pattern.y))
```

## License

MIT
