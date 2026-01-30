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


def test_01_load_pattern_gsas_raw():
    """Load a GSAS RAW file by path."""
    path = DATA_DIR / "gsas_raw" / "sample.raw"
    pattern = geddes.load_pattern(str(path))
    _assert_pattern(pattern)


def test_02_load_pattern_rasx():
    """Load a RASX archive by path."""
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.load_pattern(str(path))
    _assert_pattern(pattern)


def test_03_load_pattern_xrdml():
    """Load an XRDML file by path."""
    path = DATA_DIR / "xrdml" / "sample.xrdml"
    pattern = geddes.load_pattern(str(path))
    _assert_pattern(pattern)


def test_04_load_pattern_xy():
    """Load an XY file by path."""
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.load_pattern(str(path))
    _assert_pattern(pattern)


def test_05_load_pattern_csv():
    """Load a CSV file by path."""
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.load_pattern(str(path))
    _assert_pattern(pattern)


def test_06_load_pattern_from_bytes_gsas_raw():
    """Load GSAS RAW data from bytes."""
    path = DATA_DIR / "gsas_raw" / "sample.raw"
    data = path.read_bytes()
    pattern = geddes.load_pattern_from_bytes(data, "sample.raw")
    _assert_pattern(pattern)


def test_07_load_pattern_from_bytes_rasx():
    """Load RASX data from bytes."""
    path = DATA_DIR / "rasx" / "sample.rasx"
    data = path.read_bytes()
    pattern = geddes.load_pattern_from_bytes(data, "sample.rasx")
    _assert_pattern(pattern)


def test_08_load_pattern_from_bytes_xrdml():
    """Load XRDML data from bytes."""
    path = DATA_DIR / "xrdml" / "sample.xrdml"
    data = path.read_bytes()
    pattern = geddes.load_pattern_from_bytes(data, "sample.xrdml")
    _assert_pattern(pattern)


def test_09_load_pattern_from_bytes_xy():
    """Load XY data from bytes."""
    path = DATA_DIR / "xy" / "sample.xy"
    data = path.read_bytes()
    pattern = geddes.load_pattern_from_bytes(data, "sample.xy")
    _assert_pattern(pattern)


def test_10_load_pattern_from_bytes_csv():
    """Load CSV data from bytes."""
    path = DATA_DIR / "csv" / "sample.csv"
    data = path.read_bytes()
    pattern = geddes.load_pattern_from_bytes(data, "sample.csv")
    _assert_pattern(pattern)
