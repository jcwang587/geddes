# Geddes

Geddes loads XRD patterns into two arrays: **`x` is 2θ in degrees and `y` is intensity**.
A shared Rust implementation provides the same readers to Rust, Python, and Node.js.

## What you can do

| Functionality | Support |
|---|---|
| Read pattern files | ASCII, GSAS, RAW3/RAW4, RAS, RASX, UXD, BRML, XRDML, CHI, and powder CIF |
| Read in-memory data | Bytes in all three languages; seekable streams in Rust |
| Select a pattern | Scan or bank index; powder CIF data-block name |
| Detect the format | File content, with a filename hint when needed |
| Obtain a consistent result | Two nonempty, equally sized arrays of finite values, with increasing 2θ |

Geddes focuses on pattern loading. It returns the measured x/y data without
metadata or uncertainties and performs no smoothing, background subtraction,
or resampling. The [reading guide](reading-patterns.md) explains intensity units,
format corrections, and axis validation.

## Read your first pattern

Install the Python package:

```sh
pip install geddes
```

Read a file:

```python
import geddes

pattern = geddes.read("sample.xrdml")
x, y = pattern.x, pattern.y
```

See [Getting started](getting-started.md) for Rust and Node.js installation and
examples, or [Development](development.md) to build and test the source.

## Explore the documentation

| Guide | Contents |
|---|---|
| [Getting started](getting-started.md) | Installation and first examples in each language |
| [Supported formats](formats.md) | Format variants, extensions, and limits |
| [Reading patterns](reading-patterns.md) | Files, bytes, scan selection, intensity handling, and validation |
| [API reference](api.md) | Functions, options, return values, and migration |
| [Development](development.md) | Tests, sample patterns, benchmarks, and documentation builds |
