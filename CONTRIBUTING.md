# Contributing to Geddes

Run development commands from the Geddes repository root. The test suite checks
file and byte loading, selections, format variants, malformed inputs, and the
x/y-only API.

## Run the tests

### Python

Rebuild the local extension in an activated Python environment, then run its tests:

```sh
python -m pip install -e ".[test]"
python -m pytest tests/test_python.py -q
```

### Rust

```sh
cargo test
```

### Node.js

```sh
npm --prefix node install
npm --prefix node run build
node node/test.cjs
```

To test a separately built binding, set
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

The separate `geddes-test` repository consumes the same manifest. Its README
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

### What the timing measures

Repeated calls measure warm OS-cache performance, including language binding
overhead. They do not measure cold disk access. Imports, reference loading,
correctness checks, plotting, and harness logging are outside the timer.
Richer reader APIs may also calculate metadata or uncertainties, so timing
comparisons describe each tool's public loading workload.

Small synthetic profiles primarily expose fixed overhead; larger experimental
files exercise parsing and decompression. Interpret ratios per file and format,
and inspect exclusions before comparing timings.

## Preview the documentation

The documentation site uses Zensical. Create a dedicated environment and install
the pinned documentation dependencies:

### macOS / Linux

```sh
python -m venv .venv-docs
.venv-docs/bin/python -m pip install -r requirements-docs.txt
.venv-docs/bin/zensical serve
```

```sh
.venv-docs/bin/zensical build --strict
```

### Windows (PowerShell)

```powershell
python -m venv .venv-docs
.venv-docs\Scripts\python.exe -m pip install -r requirements-docs.txt
.venv-docs\Scripts\zensical.exe serve
```

```powershell
.venv-docs\Scripts\zensical.exe build --strict
```

The generated site is written to `site/`. The documentation workflow runs the
same strict build and uploads the site as an artifact. Keep generated output
and the documentation environment out of version control.

## Publish the npm packages

Users install `geddes` alone. Its compiled bindings are five platform packages
under the `@geddes-xrd` scope, named by `napi.packageName` in
`node/package.json`:

| Target | Package |
|---|---|
| `x86_64-pc-windows-msvc` | `@geddes-xrd/geddes-win32-x64-msvc` |
| `aarch64-pc-windows-msvc` | `@geddes-xrd/geddes-win32-arm64-msvc` |
| `aarch64-apple-darwin` | `@geddes-xrd/geddes-darwin-arm64` |
| `x86_64-unknown-linux-gnu` | `@geddes-xrd/geddes-linux-x64-gnu` |
| `aarch64-unknown-linux-gnu` | `@geddes-xrd/geddes-linux-arm64-gnu` |

`geddes` lists them as optional dependencies, so npm installs only the package
for the user's system, and the loader in `geddes` requires it. Releases up to
1.1.0 depend on unscoped `geddes-<platform>` packages instead.

The [npm publishing workflow](https://github.com/jcwang587/geddes/blob/main/.github/workflows/publish-npm.yml)
runs on release tags and publishes the tested tarballs through npm trusted
publishing, platform packages first and `geddes` last. Before it uploads
anything, it looks up every package on the registry. It stops if a package
does not exist yet, or if a version already on npm differs from the tested
tarball.

### Set up a new platform package

npm configures trusted publishing only for a package that already exists.
Before the first release that includes a new platform package, publish a
placeholder under its name by hand, then allow the publishing workflow to
publish it. Use an npm account with two-factor authentication that can publish
to the `geddes-xrd` organization. For the five packages above:

```sh
for platform in win32-x64-msvc win32-arm64-msvc darwin-arm64 linux-x64-gnu linux-arm64-gnu; do
  name="@geddes-xrd/geddes-$platform"
  placeholder="$(mktemp -d)"
  printf '{"name": "%s", "version": "0.0.0", "license": "MIT"}\n' "$name" > "$placeholder/package.json"
  npm publish "$placeholder" --access public
  npm trust github "$name" --repo jcwang587/geddes --file publish-npm.yml --allow-publish
done
```

The placeholder version, `0.0.0`, is below every release. Do not publish a
release version by hand: the workflow skips a version already on npm only when
its tarball matches the tested one byte for byte. A scoped package is private
unless it is published with `--access public`.

If a release's publishing run stopped because a package did not exist yet,
complete the setup, then publish that run's tested tarballs:

```sh
gh workflow run publish-npm.yml --ref vX.Y.Z -f publishToNPM=yes -f sourceRunId=RUN_ID
```
