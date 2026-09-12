# API reference

Read functions return one `Pattern` with nonempty, equally sized `x` and `y`
arrays. Positions are 2theta in degrees; intensities follow the
[file's conventions](reading-patterns.md#intensity-values).

`index` defaults to `0` and selects a scan, range, or bank by its zero-based
position in file order.
`block` selects a powder CIF block by a case-sensitive name substring; the
default is the first block containing a profile. CIF and single-pattern formats
require `index=0`. Byte loaders use `filename` as a format hint without opening it.

## Python

```python
geddes.read(path, *, index=0, block=None)
geddes.read_bytes(data, filename, *, index=0, block=None)
geddes.Pattern(x, y)
```

`path` and `filename` are strings; `data` is `bytes`. Pass `str(path)` for a path
object. `index` is a nonnegative integer and `block` is a string or `None`.
The returned pattern exposes `x` and `y` as Python lists. The constructor accepts
two numeric sequences and requires finite values with strictly increasing x.

File I/O failures raise `OSError`; format, parsing, and validation failures raise
`ValueError`. Invalid argument types can raise `TypeError`.
`geddes.__version__` gives the installed version.

## Rust

```rust
pub struct Pattern {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

pub struct ReadOptions {
    pub index: usize,
    pub block: Option<String>,
}
```

| Function | Input |
|---|---|
| `read(path)` | File path |
| `read_with_options(path, options)` | File path and selection |
| `read_bytes(bytes, filename)` | Complete file bytes |
| `read_bytes_with_options(bytes, filename, options)` | Bytes and selection |
| `from_reader(reader, filename)` | Readable stream |
| `from_reader_with_options(reader, filename, options)` | Stream and selection |
| `Pattern::new(x, y)` | Two `Vec<f64>` arrays |

Paths accept `impl AsRef<Path>`, bytes accept `impl AsRef<[u8]>`, and streams
require `Read + Seek`. `filename` is `&str`; `options` is `&ReadOptions`.
`ReadOptions::default()` selects index `0` with no block filter.

All functions above return `Result<Pattern, geddes::Error>`. The non-exhaustive
error enum covers I/O, ZIP, format, and parsing failures. `Pattern::new` requires
finite values with strictly increasing x. Direct field mutation and Serde
deserialization do not run its validation.

## Node.js

```typescript
interface Pattern { x: number[]; y: number[] }
interface ReadOptions { index?: number; block?: string }

function read(path: string, options?: ReadOptions): Pattern
function readBytes(
  data: Buffer, filename: string, options?: ReadOptions
): Pattern
```

Both functions are synchronous and throw on loading or parsing failures. Use a
nonnegative integer representable as an unsigned 32-bit value for `index`.
The result is a plain object; there is no Node.js `Pattern` constructor.

For migration from the uncertainty API, remove `e` access and the third
constructor argument. Patterns and serialized results contain only `x` and `y`.
