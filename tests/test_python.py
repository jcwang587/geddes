import math
from pathlib import Path

import geddes
import pytest

ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "tests" / "data"
FLOAT32_MIN_POSITIVE = float.fromhex("0x1.0p-126")


def _assert_pattern(pattern):
    assert len(pattern.x) > 0
    assert len(pattern.x) == len(pattern.y)
    if pattern.e is not None:
        assert len(pattern.e) == len(pattern.x)


def test_01_read_gsas_raw():
    path = DATA_DIR / "gsas_raw" / "gsas.raw"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_02_read_bruker_raw():
    path = DATA_DIR / "bruker_raw" / "bruker4_v5converter.raw"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_03_read_rasx():
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_04_read_xrdml():
    path = DATA_DIR / "xrdml" / "sample.xrdml"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_05_read_xy():
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_06_read_csv():
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_07_read_bytes_gsas_raw():
    path = DATA_DIR / "gsas_raw" / "gsas.raw"
    pattern = geddes.read_bytes(path.read_bytes(), "gsas.raw")
    _assert_pattern(pattern)


def test_08_read_bytes_bruker_raw():
    path = DATA_DIR / "bruker_raw" / "bruker4_v5converter.raw"
    pattern = geddes.read_bytes(path.read_bytes(), "bruker4_v5converter.raw")
    _assert_pattern(pattern)


def test_09_read_bytes_rasx():
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.rasx")
    _assert_pattern(pattern)


def test_10_read_bytes_xrdml():
    path = DATA_DIR / "xrdml" / "sample.xrdml"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.xrdml")
    _assert_pattern(pattern)


def test_11_read_bytes_xy():
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.xy")
    _assert_pattern(pattern)


def test_12_read_bytes_csv():
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.csv")
    _assert_pattern(pattern)


def test_13_bruker_raw_axis_span_is_physical():
    path = DATA_DIR / "bruker_raw" / "bruker4_v5converter.raw"
    pattern = geddes.read(str(path))
    assert len(pattern.x) == len(pattern.y)
    assert len(pattern.x) > 10

    x_start = pattern.x[0]
    x_end = pattern.x[-1]
    assert math.isfinite(x_start) and math.isfinite(x_end)
    assert x_end > x_start, f"Bruker x axis must be increasing: {x_start} -> {x_end}"


def test_14_bruker_raw_diffrac_eva_loads_with_axis():
    path = DATA_DIR / "bruker_raw" / "bruker4_diffrac_eva.raw"
    pattern = geddes.read(str(path))
    assert len(pattern.x) == len(pattern.y)
    assert len(pattern.x) > 10

    x_start = pattern.x[0]
    x_end = pattern.x[-1]
    assert math.isfinite(x_start) and math.isfinite(x_end)
    assert x_end > x_start, f"Bruker x axis must be increasing: {x_start} -> {x_end}"

    subnormal = sum(
        1 for value in pattern.y if value != 0.0 and abs(value) < FLOAT32_MIN_POSITIVE
    )
    ratio = subnormal / len(pattern.y)
    assert ratio < 0.05, f"Too many subnormal intensity values: ratio={ratio}"


def test_15_pattern_new_rejects_mismatched_xy_lengths():
    with pytest.raises(ValueError, match="x and y must have the same length"):
        geddes.Pattern([10.0], [100.0, 101.0], None)


def test_16_pattern_new_rejects_non_ascending_x():
    with pytest.raises(ValueError, match="x values must be strictly increasing"):
        geddes.Pattern([20.0, 10.0], [100.0, 101.0], None)


def test_17_read_bytes_rejects_descending_xrdml_axis():
    data = b"""<?xml version="1.0" encoding="UTF-8"?>
<xrdMeasurements xmlns="http://www.xrdml.com/XRDMeasurement/1.6">
  <xrdMeasurement>
    <scan>
      <dataPoints>
        <positions axis="2Theta">
          <startPosition>20.0</startPosition>
          <endPosition>10.0</endPosition>
        </positions>
        <intensities>100 101 102</intensities>
      </dataPoints>
    </scan>
  </xrdMeasurement>
</xrdMeasurements>
"""

    with pytest.raises(ValueError, match="x values must be strictly increasing"):
        geddes.read_bytes(data, "descending.xrdml")
