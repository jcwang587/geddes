# Geddes

Geddes reads XRD files into `x` (2θ in degrees) and `y` (intensity).
It is available for Python, Rust, and Node.js.

## Installation and usage

Save the [toy pattern](assets/samples/profile.xy) in your working directory,
or use the path to your own file.

=== "Python"

    Python 3.10 or later:

    ```sh
    pip install geddes
    ```

    ```python
    import geddes

    pattern = geddes.read("profile.xy")
    x, y = pattern.x, pattern.y
    ```

=== "Rust"

    ```sh
    cargo add geddes
    ```

    ```rust
    fn main() -> Result<(), geddes::Error> {
        let pattern = geddes::read("profile.xy")?;
        println!("{:?}\n{:?}", pattern.x, pattern.y);
        Ok(())
    }
    ```

=== "Node.js"

    Node.js 16 or later:

    ```sh
    npm install @jcwang587/geddes
    ```

    ```javascript
    const geddes = require('@jcwang587/geddes')

    const { x, y } = geddes.read('profile.xy')
    ```

The sample returns `x = [10, 11, 12]` and `y = [4, 9, 16]`.
Each call reads one pattern. See [usage](reading-patterns.md) for pattern selection
and [formats](formats.md) for supported files.
