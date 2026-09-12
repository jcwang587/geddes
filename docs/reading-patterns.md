# Usage

## Pattern selection

| Option | Meaning | Default |
|---|---|---|
| `index` | Zero-based position of a scan, range, or GSAS bank | `0` (first pattern) |
| `block` | Case-sensitive substring of a powder CIF block name | First block containing a profile |

Select the second scan or a measured CIF block:

=== "Python"

    ```python
    import geddes

    pattern = geddes.read("measurement.rasx", index=1)
    pattern = geddes.read("standard.cif", block="_meas")
    ```

=== "Rust"

    ```rust
    fn main() -> Result<(), geddes::Error> {
        let options = geddes::ReadOptions { index: 1, block: None };
        let pattern = geddes::read_with_options("measurement.rasx", &options)?;

        let options = geddes::ReadOptions {
            block: Some("_meas".into()), ..Default::default()
        };
        let pattern = geddes::read_with_options("standard.cif", &options)?;
        Ok(())
    }
    ```

=== "Node.js"

    ```javascript
    const geddes = require('@jcwang587/geddes')

    const pattern = geddes.read('measurement.rasx', { index: 1 })
    const measured = geddes.read('standard.cif', { block: '_meas' })
    ```

Indices follow file order. An out-of-range index or unmatched block raises an error. CIF and
single-pattern formats require `index=0`; a CIF substring selects the first
matching profile block.

## Reading bytes

Pass the file contents and a filename hint. The filename does not open a file.

=== "Python"

    ```python
    import geddes

    pattern = geddes.read_bytes(b"10 4\n11 9\n12 16\n", "sample.xy")
    ```

=== "Rust"

    ```rust
    fn main() -> Result<(), geddes::Error> {
        let pattern = geddes::read_bytes(b"10 4\n11 9\n12 16\n", "sample.xy")?;
        Ok(())
    }
    ```

=== "Node.js"

    ```javascript
    const geddes = require('@jcwang587/geddes')

    const data = Buffer.from('10 4\n11 9\n12 16\n')
    const pattern = geddes.readBytes(data, 'sample.xy')
    ```

Byte readers accept the same selection options. Rust also supports
`Read + Seek` streams through [`from_reader`](api.md#rust).

## Intensity values

Intensities retain their stored units. Counts per second are not converted to
counts.

| Input | Intensity handling |
|---|---|
| XRDML raw `<counts>` | Multiplied by supplied attenuation factors |
| XRDML processed `<intensities>` and BRML | Returned as stored |
| Rigaku | Additional attenuator column ignored |

Geddes returns x/y only. It does not smooth, resample, subtract backgrounds,
calculate uncertainties, or remove points with zero uncertainty or weight.

## Ordering and validation

- **Arrays:** Finite, nonempty, and equally sized.
- **Strictly descending scans:** Reversed as paired x/y arrays.
- **Invalid axes:** Duplicate positions or changes in direction raise errors.
- **Constructors:** Direct `Pattern` constructors require increasing x.

Geddes does not convert Q, d-spacing, or time-of-flight axes. See the
[format limits](formats.md#limits) for inputs without a reliable axis declaration.

## Using the arrays

These Python examples use the [toy pattern](assets/samples/profile.xy).
Install Matplotlib and NumPy for plotting and CSV export:

```sh
pip install matplotlib numpy
```

Plot x and y:

```python
import geddes
import matplotlib.pyplot as plt

pattern = geddes.read("profile.xy")
plt.plot(pattern.x, pattern.y, "o-")
plt.xlabel("2θ (degrees)")
plt.ylabel("Intensity")
plt.show()
```

Save a two-column CSV:

```python
import geddes
import numpy as np

pattern = geddes.read("profile.xy")
xy = np.column_stack((pattern.x, pattern.y))
np.savetxt("profile.csv", xy, delimiter=",", header="2theta,intensity", comments="")
```
