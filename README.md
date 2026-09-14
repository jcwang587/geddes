# Geddes

[![Crates.io](https://img.shields.io/crates/v/geddes?logo=rust&color=264323)](https://crates.io/crates/geddes)
[![PyPI](https://img.shields.io/pypi/v/geddes?logo=pypi&logoColor=white&color=3775A9)](https://pypi.org/project/geddes/)
[![npm](https://img.shields.io/npm/v/%40jcwang587%2Fgeddes?logo=npm&color=CB3837)](https://www.npmjs.com/package/@jcwang587/geddes)

Geddes reads XRD files into two arrays: `x` (2θ in degrees) and `y` (intensity).
It is written in Rust, with Python and Node.js bindings.

## Quick start

### Python

```python
import geddes

pattern = geddes.read("sample.xy")
x, y = pattern.x, pattern.y
```

### Rust

```rust
fn main() -> Result<(), geddes::Error> {
    let pattern = geddes::read("sample.xy")?;
    let (x, y) = (pattern.x, pattern.y);
    Ok(())
}
```

### Node.js

```javascript
const geddes = require('@jcwang587/geddes')

const { x, y } = geddes.read('sample.xy')
```

## Supported formats

| Format | Extensions |
|---|---|
| Text | `.xy`, `.xye`, `.csv`, `.dat`, `.prn`, `.txt` |
| GSAS | `.gsas`, `.gsa`, `.fxye`, `.gda`, `.xra`, `.raw` |
| Bruker RAW | `.raw` |
| RAS | `.ras` |
| RASX | `.rasx` |
| UXD | `.uxd` |
| BRML | `.brml` |
| XRDML | `.xrdml` |
| CHI | `.chi` |
| Powder CIF | `.cif` |

See the [documentation](https://jcwang587.github.io/geddes/) for installation, usage, and format limits.
