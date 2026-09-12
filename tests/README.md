# Tests and fixture references

Run these commands from the Geddes repository root.

## Rust

```sh
cargo test
```

The suite checks the x/y-only API, text formats, Bruker binary layouts, XML/ZIP
profiles, selection and malformed inputs. Targeted text or binary regressions
can be run with:

```sh
cargo test --test text_formats
cargo test --test bruker_formats
```

The [shared fixture manifest](data/formats/manifest.json) identifies source
files, expected point counts, scan/block selections, tolerances and full-array
CSV references. These references test every x and y value rather than only
successful loading or array lengths. The [provenance record](data/formats/README.md)
describes experimental files and their licenses; synthetic fixtures specify
their intended arrays directly.

## Python

Build the local extension and run its tests:

```sh
python -m pip install -e ".[test]"
python -m pytest tests/test_python.py -q
```

Python tests cover path and byte loading, the two-argument
`geddes.Pattern(x, y)` constructor, selections, errors, and the absence of `e`.
Rebuild the extension after changing Rust source before testing Python.

## Node.js

Build the bindings from the local Rust source:

```sh
npm --prefix node install
npm --prefix node run build
node node/test.cjs
```

The Node test loads every manifest case through both path and byte APIs. To use
a separately built native binding, set its absolute path:

```sh
GEDDES_BINDING=/absolute/path/to/geddes.node node node/test.cjs
```

The Node API returns `{ x, y }`. Both `read` and `readBytes` accept an optional
`{ scan, block }` selection object; examples are in the [main README](../README.md).

## Rebuilding reference data

[build_fixture_corpus.py](build_fixture_corpus.py) is a development tool, not a
runtime dependency. Install NumPy and gemmi in a development environment, then
rebuild the committed CSV references and manifest:

```sh
python -m pip install numpy gemmi
python tests/build_fixture_corpus.py
```

It uses literal expected arrays for synthetic cases and independent,
fixture-specific extraction for real files: NumPy text loading, Python XML/ZIP
and `struct`, and gemmi's CIF grammar. It imports neither Geddes nor Rietx. These
extractors document the selected fixture layouts; they are not general-purpose
replacement readers. Inspect changes to the expected arrays when regenerating
them, since changing a reference can hide a reader regression.

To refresh vendored experimental inputs, use a Rietx checkout at commit
`88d2353f446d98632ef233437000cbe7bc4e1142`:

```sh
python tests/build_fixture_corpus.py --rietx-source /path/to/rietx
```

The command copies the listed input files and regenerates references; it does
not fetch or verify the checkout's Git revision. Preserve the provenance and
license files when updating the corpus.

## Benchmarking

Use the sibling [geddes-test repository](../../geddes-test/README.md), which
consumes this manifest. Build Geddes in release mode for timed comparisons.
The benchmark validates full arrays before measuring file loading and reports
unsupported or numerically different results separately. It does not discard
rows, interpolate, or change the selected scan to force agreement.

Keep these semantics in mind when interpreting comparisons:

- Geddes preserves x/y rows whose uncertainty or weight is zero. Rietx may
  filter such rows; a different array length is a semantic mismatch.
- XRDML raw `<counts>` receive supplied attenuation factors; processed
  `<intensities>` and BRML intensities are preserved. No counts-per-second
  normalization occurs.
- Descending profiles are reversed as paired arrays. Duplicate x values are
  rejected, not deduplicated.
- Small synthetic files often measure API overhead. Large experimental files
  exercise parsing and decompression. Scrambled RAW4 intensities can verify
  byte decoding but cannot validate a physical measurement.
