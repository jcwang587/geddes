# Reading patterns

Each read returns one pattern as paired `x` and `y` arrays. Selection changes
which pattern is loaded; it does not concatenate scans or add metadata to the
result.

## Select a scan or bank

`scan` is a zero-based index and defaults to `0`. It applies to GSAS banks,
Bruker RAW scans, RAS/RASX scans, UXD ranges, XRDML scans, and BRML scans. Formats
with one pattern require `scan=0`.

Python:

```python
import geddes

pattern = geddes.read("measurement.rasx", scan=1)
```

Rust:

```rust
let options = geddes::ReadOptions { scan: 1, block: None };
let pattern = geddes::read_with_options("measurement.rasx", &options)?;
```

Node.js:

```javascript
const pattern = geddes.read('measurement.rasx', { scan: 1 })
```

Byte-loading APIs accept the same selection. For example:

```python
from pathlib import Path

data = Path("measurement.rasx").read_bytes()
pattern = geddes.read_bytes(data, "measurement.rasx", scan=1)
```

## Select a powder CIF block

`block` is a case-sensitive substring of a data-block name. The default selects
the first block containing a recognized profile. A substring selects the first
matching block that contains a profile; choosing a name with no match is an error.

```python
pattern = geddes.read("standard.cif", block="_meas")
```

```rust
let options = geddes::ReadOptions {
    block: Some("_meas".into()),
    ..Default::default()
};
let pattern = geddes::read_with_options("standard.cif", &options)?;
```

```javascript
const pattern = geddes.read('standard.cif', { block: '_meas' })
```

## Intensity values

`y` retains the file's native intensity units. Counts per second are not
normalized to counts, and Geddes does not subtract backgrounds, smooth peaks,
resample, or calculate uncertainties.

| Input | Returned intensity |
|---|---|
| XRDML raw `<counts>` | Counts multiplied by supplied `beamAttenuationFactors` |
| XRDML processed `<intensities>` | Stored values |
| BRML | Stored intensity field; no additional absorber correction |
| RAS and RASX | Stored second column; extra attenuator column ignored |
| Other supported profiles | Stored intensity values |

Extra uncertainty or weight columns are not returned. A zero uncertainty or
weight does not discard its x/y point.

## Ordering and validation

Returned arrays are nonempty, equally sized and finite. `x` is strictly
increasing. When a file contains a strictly descending scan, loading reverses
both arrays together to preserve each intensity's position.

Duplicate x values and axes that change direction are errors. Geddes does not
sort, deduplicate, or merge them. Direct `Pattern` constructors also validate the
arrays, but require the caller to provide an already increasing x axis.

Format-specific checks reject recognized non-2theta axes. Check the
[format limits](formats.md#limits) when reading data without an explicit axis
declaration. [API reference](api.md) describes each binding's error behavior.
