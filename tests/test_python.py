from pathlib import Path

import pytest

import geddes

ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "tests" / "data"


def _assert_pattern(pattern):
    assert len(pattern.x) > 0
    assert len(pattern.x) == len(pattern.y)
    if pattern.e is not None:
        assert len(pattern.e) == len(pattern.x)


def test_load_file_xy():
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_file_gsas_raw():
    path = DATA_DIR / "gsas_raw" / "sample.raw"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_file_rasx():
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_file_csv():
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.load_file(str(path))
    _assert_pattern(pattern)


def test_load_bytes_xy():
    path = DATA_DIR / "xy" / "sample.xy"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.xy")
    _assert_pattern(pattern)


def test_load_bytes_gsas_raw():
    path = DATA_DIR / "gsas_raw" / "sample.raw"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.raw")
    _assert_pattern(pattern)


def test_load_bytes_rasx():
    path = DATA_DIR / "rasx" / "sample.rasx"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.rasx")
    _assert_pattern(pattern)


def test_load_bytes_csv():
    path = DATA_DIR / "csv" / "sample.csv"
    data = path.read_bytes()
    pattern = geddes.load_bytes(data, "sample.csv")
    _assert_pattern(pattern)
