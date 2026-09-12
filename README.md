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

| Format | Extensions | Pattern selection |
|---|---|---|
| ASCII columns | `.xy`, `.xye`, `.csv`, `.dat`, `.prn`, `.txt` | Single pattern |
| GSAS STD / ESD / FXYE | `.gsas`, `.gsa`, `.fxye`, `.gda`, `.xra`, `.raw` | Bank index |
| Bruker RAW3 / RAW4 | `.raw` | Scan index |
| Rigaku RAS | `.ras` | Scan index |
| Rigaku RASX | `.rasx` | Scan index |
| Bruker/Siemens UXD | `.uxd` | Scan index |
| Bruker BRML | `.brml` | Scan index |
| PANalytical XRDML | `.xrdml` | Scan index |
| FIT2D/pyFAI CHI | `.chi` | Single pattern |
| Powder CIF | `.cif` | Data-block name |

See the [documentation](docs/index.md) for installation, usage, and format limits.
