# Getting started

Geddes reads an XRD pattern into two arrays: `x` contains 2theta in degrees and
`y` contains intensity. Choose the binding for your project below.

## Python

Install Geddes with Python 3.10 or later:

```sh
python -m pip install geddes
```

Load a file:

```python
import geddes

pattern = geddes.read("sample.xrdml")
print(pattern.x)
print(pattern.y)
```

For data already in memory, pass bytes and a filename hint:

```python
pattern = geddes.read_bytes(b"10 4\n11 9\n12 16\n", "sample.xy")
assert pattern.x == [10.0, 11.0, 12.0]
assert pattern.y == [4.0, 9.0, 16.0]
```

## Rust

Add Geddes to your project:

```sh
cargo add geddes
```

Load a file with `read`, or use `read_bytes` for an in-memory buffer:

```rust
fn main() -> Result<(), geddes::Error> {
    let pattern = geddes::read("sample.xrdml")?;
    println!("{} points", pattern.x.len());

    let pattern = geddes::read_bytes(b"10 4\n11 9\n12 16\n", "sample.xy")?;
    assert_eq!(pattern.x, vec![10.0, 11.0, 12.0]);
    assert_eq!(pattern.y, vec![4.0, 9.0, 16.0]);
    Ok(())
}
```

## Node.js

Install the binding with Node.js 16 or later:

```sh
npm install @jcwang587/geddes
```

Use `read` for a file or `readBytes` for a `Buffer`:

```javascript
const geddes = require('@jcwang587/geddes')

const pattern = geddes.read('sample.xrdml')
console.log(pattern.x, pattern.y)

const fromBytes = geddes.readBytes(
  Buffer.from('10 4\n11 9\n12 16\n'),
  'sample.xy'
)
console.log(fromBytes.x, fromBytes.y)
```

## Choose what to read

Every call loads one pattern. The default is the first scan or the first powder
CIF block containing a recognized profile. See [reading patterns](reading-patterns.md)
to select another scan or block, and [supported formats](formats.md) for file
variants and limits. The [API reference](api.md) lists all entry points.
