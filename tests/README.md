# Test Run Guide

Run commands from the repository root (`geddes/`).

## Rust tests

Run the Rust loader test suite:

```bash
cargo test --test loader_tests -- --nocapture
```

Run a single Rust test:

```bash
cargo test --test loader_tests test_14_bruker_raw_diffrac_eva_loads_with_axis -- --nocapture
```

## Python tests

Install the Python extension module and test dependency:

```bash
python -m pip install -e ".[test]"
```

Run the Python test file:

```bash
python -m pytest tests/test_python.py -q
```

Run a single Python test:

```bash
python -m pytest tests/test_python.py -k test_03_read_xrdml -q
```
