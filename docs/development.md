# Development

Run development commands from the Geddes repository root. The test suite checks
file and byte loading, selections, format variants, malformed inputs, and the
x/y-only API.

## Run the tests

Rust:

```sh
cargo test
```

Python, after rebuilding the local extension:

```sh
python -m pip install -e ".[test]"
python -m pytest tests/test_python.py -q
```

Node.js:

```sh
npm --prefix node install
npm --prefix node run build
node node/test.cjs
```

To test a separately built Node binding, set
`GEDDES_BINDING=/absolute/path/to/geddes.node` before the Node command.
See the [repository test guide](https://github.com/jcwang587/geddes/blob/main/tests/README.md)
for more focused checks.

## Fixture corpus

The [fixture manifest](https://github.com/jcwang587/geddes/blob/main/tests/data/formats/manifest.json)
records each file, scan or CIF block, expected point count, complete reference
arrays, tolerances, and SHA-256 hashes. Tests compare every x and y value.
Synthetic cases have explicitly specified arrays; experimental files exercise
larger patterns and instrument-specific layouts.

The [provenance record](https://github.com/jcwang587/geddes/blob/main/tests/data/formats/README.md)
identifies original sources and licenses. Experimental fixtures were collected
through [Rietx's pinned test corpus](https://github.com/yue-here/rietx/tree/88d2353f446d98632ef233437000cbe7bc4e1142/tests/data),
with source credit to APS/GSAS-II tutorials, NIST, NIMS, and FAIRmat. Geddes's
authored fixtures are MIT licensed; imported files retain their source terms.

The corpus preserves measured x/y rows even when a file has a zero uncertainty.
For example, `11BM_NAC.fxye` contains 59,498 explicit rows despite its header's
59,497 channel count. Its reference retains all 59,498 rows. Scrambled RAW4
intensities test byte decoding and throughput, not physical-profile correctness.

## Regenerate references

[build_fixture_corpus.py](https://github.com/jcwang587/geddes/blob/main/tests/build_fixture_corpus.py)
is a development tool requiring NumPy and gemmi:

```sh
python -m pip install numpy gemmi
python tests/build_fixture_corpus.py
```

It uses literal synthetic arrays and independent, fixture-specific extraction
with NumPy, Python XML/ZIP/`struct`, and gemmi's CIF grammar. It imports neither
Geddes nor Rietx. Review regenerated array changes before accepting them; the
reference needs to remain independent of the implementation being tested.

To refresh the experimental inputs as well, supply a Rietx checkout at commit
`88d2353f446d98632ef233437000cbe7bc4e1142`:

```sh
python tests/build_fixture_corpus.py --rietx-source /path/to/rietx
```

This copies the listed files and rebuilds references. It does not fetch or check
the checkout's Git revision. Preserve provenance and license notices when
updating the corpus.

## Measure loading performance

The separate [geddes-test repository](https://github.com/jcwang587/geddes-test)
consumes the same manifest. Its
[benchmark guide](https://github.com/jcwang587/geddes-test/blob/main/README.md)
describes environment setup and report reproduction. Build Geddes in release
mode before timing it.

The benchmark validates full arrays before timing each reader. Unsupported
inputs, wrong selections, and differing numerical results are reported
separately and excluded from speed ratios. It does not interpolate, drop rows,
or substitute a scan to make results agree.

| Measurement | Timed work |
|---|---|
| Native API | Open, read, decompress when needed, parse, and return the native result |
| NumPy x/y | The same call plus extracting x/y and creating NumPy float64 arrays |
| Bytes only | Open and read the file without parsing |

Imports, reference loading, correctness checks, plotting, and logging are outside
the timer. Repeated calls measure warm OS-cache performance, including language
binding overhead. They do not measure cold disk access. Richer reader APIs may
also calculate metadata or uncertainties, so timing comparisons describe each
tool's public loading workload.

Small synthetic profiles primarily expose fixed overhead; larger experimental
files exercise parsing and decompression. Interpret ratios per file and format,
and inspect exclusions before comparing timings.

## Preview the documentation

The documentation site uses Zensical. Create a dedicated environment and install
the pinned documentation dependencies:

```sh
python -m venv .venv-docs
.venv-docs/bin/python -m pip install -r requirements-docs.txt
.venv-docs/bin/zensical serve
```

Build the site without starting the preview server:

```sh
.venv-docs/bin/zensical build --strict
```
