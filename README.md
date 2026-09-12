# Geddes

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

| Format | Extensions | Selection |
|---|---|---|
| Text | `.xy`, `.xye`, `.csv`, `.dat`, `.prn`, `.txt` | Single pattern |
| GSAS | `.gsas`, `.gsa`, `.fxye`, `.gda`, `.xra`, `.raw` | Bank index |
| Bruker RAW | `.raw` (v3, v4) | Scan index |
| RAS | `.ras` | Scan index |
| RASX | `.rasx` | Scan index |
| UXD | `.uxd` | Scan index |
| BRML | `.brml` | Scan index |
| XRDML | `.xrdml` | Scan index |
| CHI | `.chi` | Single pattern |
| Powder CIF | `.cif` | Data-block name |

See the [documentation](docs/index.md) for installation, usage, and format limits.
