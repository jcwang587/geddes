# Geddes

[![Crates.io](https://img.shields.io/crates/v/geddes)](https://crates.io/crates/geddes)
[![PyPI](https://img.shields.io/pypi/v/geddes)](https://pypi.org/project/geddes/)


A Rust XRD pattern parser with Python bindings. Supports:
- `.raw` (GSAS text or Bruker binary)
- `.rasx` (Rigaku Zip archive)
- `.xrdml` (Panalytical XML)
- `.xy` / `.xye` (Space-separated ASCII)
- `.csv` (Comma-separated values)

## Usage

Load from a file path:

```rust
use geddes::read;

fn main() {
    let pattern = read("tests/data/xy/sample.xy").unwrap();
    println!("{} {}", pattern.x.len(), pattern.y.len());
}
```

Load from in-memory bytes (filename is used to infer the format):

```rust
use std::fs;

use geddes::read_bytes;

fn main() {
    let data = fs::read("tests/data/xy/sample.xy").unwrap();
    let pattern = read_bytes(&data, "sample.xy").unwrap();
    println!("{} {}", pattern.x.len(), pattern.y.len());
}
```

## Python Usage

Load from a file path:

```python
import geddes

pattern = geddes.read("tests/data/xy/sample.xy")
print(len(pattern.x), len(pattern.y))
```

Load from in-memory bytes (filename is used to infer the format):

```python
import geddes

with open("tests/data/xy/sample.xy", "rb") as f:
    data = f.read()

pattern = geddes.read_bytes(data, "sample.xy")
print(len(pattern.x), len(pattern.y))
```

## Node.js Usage

Load from a file path:

```javascript
const geddes = require('@jcwang587/geddes')

const pattern = geddes.read('tests/data/xy/sample.xy')
console.log(pattern.x.length, pattern.y.length)
```

Load from in-memory bytes (filename is used to infer the format):

```javascript
const fs = require('node:fs')
const geddes = require('@jcwang587/geddes')

const bytes = fs.readFileSync('tests/data/xy/sample.xy')
const pattern = geddes.readBytes(bytes, 'sample.xy')
console.log(pattern.x.length, pattern.y.length)
```

## License

MIT
