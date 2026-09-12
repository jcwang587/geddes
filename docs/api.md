# API reference

All bindings return a `Pattern` with two fields:

| Field | Meaning |
|---|---|
| `x` | Finite, strictly increasing 2theta values in degrees |
| `y` | Finite intensity values paired with `x` |

The arrays have the same nonzero length. See [reading patterns](reading-patterns.md)
for intensity conventions, scan selection, and ordering.

## Python

```python
geddes.read(path, *, scan=0, block=None)
geddes.read_bytes(data, filename, *, scan=0, block=None)
geddes.Pattern(x, y)
```

| Argument | Type | Use |
|---|---|---|
| `path` | `str` | File to load |
| `data` | `bytes` | Complete file contents |
| `filename` | `str` | Format hint for byte loading |
| `scan` | Nonnegative `int` | Zero-based scan or bank index |
| `block` | `str` or `None` | Powder CIF block-name substring |

`read` and `read_bytes` return a `geddes.Pattern`. Its `x` and `y` properties
provide Python lists. `geddes.Pattern(x, y)` constructs a validated pattern from
two sequences of numbers.

File I/O failures raise `OSError`; format, parsing, and array-validation failures
raise `ValueError`. Invalid Python argument types can raise `TypeError`.
`geddes.__version__` gives the installed package version.

## Rust

```rust
pub struct Pattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

pub struct ReadOptions {
    pub scan: usize,
    pub block: Option<String>,
}
```

`ReadOptions::default()` selects scan `0` and leaves `block` unset.

| Function | Inputs |
|---|---|
| `read(path)` | `path: impl AsRef<Path>` |
| `read_with_options(path, options)` | Path and `&ReadOptions` |
| `read_bytes(bytes, filename)` | `bytes: impl AsRef<[u8]>`, `filename: &str` |
| `read_bytes_with_options(bytes, filename, options)` | Bytes, filename hint, and `&ReadOptions` |
| `from_reader(reader, filename)` | `reader: impl Read + Seek`, `filename: &str` |
| `from_reader_with_options(reader, filename, options)` | Reader, filename hint, and `&ReadOptions` |
| `Pattern::new(x, y)` | Two `Vec<f64>` arrays with increasing x |

Each function returns `Result<Pattern, geddes::Error>`. `Error` is a
non-exhaustive enum covering file I/O, ZIP, parsing, unknown-format, and missing
archive-member failures. `Pattern` supports cloning, debug output, and Serde
serialization and deserialization. Serialized results contain only `x` and `y`.

The constructor and read APIs validate arrays. Rust fields are public, so direct
struct construction, later mutation, and Serde deserialization do not apply
those constructor checks automatically.

## Node.js

```typescript
interface Pattern {
  x: number[]
  y: number[]
}

interface ReadOptions {
  scan?: number
  block?: string
}

function read(path: string, options?: ReadOptions): Pattern
function readBytes(
  data: Buffer,
  filename: string,
  options?: ReadOptions
): Pattern
```

Omitting `options` selects the first pattern. `scan` must be a nonnegative
integer representable as an unsigned 32-bit value. Both functions are
synchronous and throw on loading or parsing failures. The result is a plain
object containing `x` and `y`; there is no Node.js `Pattern` constructor.

## Migration from the uncertainty API

The x/y-only API removes `e` from Rust, Python, Node.js, and serialized results.
Remove uncertainty-field access and the third constructor argument:

```python
pattern = geddes.Pattern(x, y)
```

```rust
let pattern = geddes::Pattern::new(x, y)?;
```

Input files may still carry uncertainty or other extra columns. Geddes reads the
profile's position and intensity without exposing those additional values or
performing uncertainty-based filtering.
