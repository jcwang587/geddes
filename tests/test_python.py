from pathlib import Path

import pytest

import geddes

ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "tests" / "data"


def _assert_pattern(pattern):
    """Validate basic invariants for a loaded pattern object."""
    assert len(pattern.x) > 0
    assert len(pattern.x) == len(pattern.y)
    if pattern.e is not None:
        assert len(pattern.e) == len(pattern.x)


def test_load_file_xy():
    """Load an XY file by path."""
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_file_gsas_raw():
    """Load a GSAS RAW file by path."""
    path = DATA_DIR / "gsas_raw" / "sample.raw"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_file_rasx():
    """Load a RASX archive by path."""
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_file_csv():
    """Load a CSV file by path."""
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_bytes_xy():
    """Load XY data from bytes."""
    path = DATA_DIR / "xy" / "sample.xy"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.xy")
    _assert_pattern(pattern)


def test_load_bytes_gsas_raw():
    """Load GSAS RAW data from bytes."""
    path = DATA_DIR / "gsas_raw" / "sample.raw"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.raw")
    _assert_pattern(pattern)


def test_load_bytes_rasx():
    """Load RASX data from bytes."""
    path = DATA_DIR / "rasx" / "sample.rasx"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.rasx")
    _assert_pattern(pattern)


def test_load_bytes_csv():
    """Load CSV data from bytes."""
    path = DATA_DIR / "csv" / "sample.csv"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.csv")
    _assert_pattern(pattern)
