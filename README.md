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

## Python Usage

This crate ships Python bindings via `pyo3`/`maturin`.

From the repo root:

```sh
pip install maturin
maturin develop
```

Then in Python:

```python
import geddes

pattern = geddes.load_file("tests/data/xy/sample.xy")
print(len(pattern.x), len(pattern.y))
```

Loading from bytes:

```python
import geddes

with open("tests/data/xy/sample.xy", "rb") as f:
    data = f.read()

pattern = geddes.load_bytes(data, "sample.xy")
print(len(pattern.x))
```

## License

MIT
