# Geddes

[![Crates.io](https://img.shields.io/crates/v/geddes)](https://crates.io/crates/geddes)
[![PyPI](https://img.shields.io/pypi/v/geddes)](https://pypi.org/project/geddes/)
[![npm](https://img.shields.io/npm/v/%40jcwang587%2Fgeddes)](https://www.npmjs.com/package/@jcwang587/geddes)

Geddes reads XRD patterns into two arrays: **`x` is 2θ in degrees and `y` is intensity**.
It is written in Rust, with Python and Node.js bindings. Read from a file or bytes,
and select one pattern from files containing multiple scans or data blocks.

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

Returns x/y only, with finite values and an increasing 2θ axis. See the
[format guide](docs/formats.md) for supported layouts and the
[reading guide](docs/reading-patterns.md) for selection, intensity units, and validation.

## Quick start

```sh
pip install geddes
```

```python
import geddes

pattern = geddes.read("sample.xrdml")
x, y = pattern.x, pattern.y
```

## Documentation

Start with the [documentation](docs/index.md) for
[Python, Rust, and Node.js installation](docs/index.md),
[API reference](docs/api.md), and
[usage](docs/reading-patterns.md).

## License

[MIT](LICENSE). Imported test files retain their upstream terms; see
[fixture provenance](tests/data/formats/README.md).
