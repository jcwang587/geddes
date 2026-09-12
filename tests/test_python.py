import math
import csv
import json
from pathlib import Path

import geddes
import pytest

ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "tests" / "data"
CORPUS = DATA_DIR / "formats"
FIXTURES = json.loads((CORPUS / "manifest.json").read_text())["fixtures"]
FLOAT32_MIN_POSITIVE = float.fromhex("0x1.0p-126")


@pytest.mark.parametrize("fixture", FIXTURES, ids=lambda f: f["id"])
def test_complete_fixture_arrays_and_bytes_api(fixture):
    path = CORPUS / fixture["path"]
    with (CORPUS / fixture["reference"]).open() as stream:
        reference = list(csv.DictReader(stream))
    expected_x = [float(r["x"]) for r in reference]
    expected_y = [float(r["y"]) for r in reference]
    options = {"scan": fixture["scan"], "block": fixture["block"]}
    for pattern in (geddes.read(str(path), **options),
                    geddes.read_bytes(path.read_bytes(), path.name, **options)):
        assert not hasattr(pattern, "e")
        assert len(pattern.x) == fixture["points"]
        assert pattern.x == pytest.approx(expected_x, rel=fixture["rtol"], abs=fixture["atol"])
        assert pattern.y == pytest.approx(expected_y, rel=fixture["rtol"], abs=fixture["atol"])


def _assert_pattern(pattern):
    assert len(pattern.x) > 0
    assert len(pattern.x) == len(pattern.y)
    assert not hasattr(pattern, "e")


def test_reads_gsas_raw_from_path():
    path = DATA_DIR / "gsas_raw" / "gsas.raw"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_reads_bruker_raw_from_path():
    path = DATA_DIR / "bruker_raw" / "bruker4_v5converter.raw"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_reads_rasx_from_path():
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)
    assert len(pattern.x) == 2334
    assert abs(pattern.x[0] - 10.0) < 1e-9


def test_reads_xrdml_from_path():
    path = DATA_DIR / "xrdml" / "sample.xrdml"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_reads_xy_from_path():
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_reads_csv_from_path():
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.read(str(path))
    _assert_pattern(pattern)


def test_reads_gsas_raw_from_bytes():
    path = DATA_DIR / "gsas_raw" / "gsas.raw"
    pattern = geddes.read_bytes(path.read_bytes(), "gsas.raw")
    _assert_pattern(pattern)


def test_reads_bruker_raw_from_bytes():
    path = DATA_DIR / "bruker_raw" / "bruker4_v5converter.raw"
    pattern = geddes.read_bytes(path.read_bytes(), "bruker4_v5converter.raw")
    _assert_pattern(pattern)


def test_reads_rasx_from_bytes():
    path = DATA_DIR / "rasx" / "sample.rasx"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.rasx")
    _assert_pattern(pattern)
    assert len(pattern.x) == 2334
    assert abs(pattern.x[0] - 10.0) < 1e-9


def test_reads_xrdml_from_bytes():
    path = DATA_DIR / "xrdml" / "sample.xrdml"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.xrdml")
    _assert_pattern(pattern)


def test_reads_xy_from_bytes():
    path = DATA_DIR / "xy" / "sample.xy"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.xy")
    _assert_pattern(pattern)


def test_reads_csv_from_bytes():
    path = DATA_DIR / "csv" / "sample.csv"
    pattern = geddes.read_bytes(path.read_bytes(), "sample.csv")
    _assert_pattern(pattern)


def test_bruker_raw_axis_span_is_physical():
    path = DATA_DIR / "bruker_raw" / "bruker4_v5converter.raw"
    pattern = geddes.read(str(path))
    assert len(pattern.x) == len(pattern.y)
    assert len(pattern.x) > 10

    x_start = pattern.x[0]
    x_end = pattern.x[-1]
    assert math.isfinite(x_start) and math.isfinite(x_end)
    assert x_end > x_start, f"Bruker x axis must be increasing: {x_start} -> {x_end}"
    assert all(
        curr > prev for prev, curr in zip(pattern.x, pattern.x[1:])
    ), "Bruker x axis must be strictly increasing"


def test_bruker_raw_diffrac_eva_loads_with_axis():
    path = DATA_DIR / "bruker_raw" / "bruker4_diffrac_eva.raw"
    pattern = geddes.read(str(path))
    assert len(pattern.x) == len(pattern.y)
    # Expected point count verified against Bruker DIFFRAC.EVA export metadata.
    assert len(pattern.x) == 7134
    assert len(pattern.x) > 10

    x_start = pattern.x[0]
    x_end = pattern.x[-1]
    assert math.isfinite(x_start) and math.isfinite(x_end)
    assert x_end > x_start, f"Bruker x axis must be increasing: {x_start} -> {x_end}"
    assert all(
        curr > prev for prev, curr in zip(pattern.x, pattern.x[1:])
    ), "Bruker x axis must be strictly increasing"

    subnormal = sum(
        1 for value in pattern.y if value != 0.0 and abs(value) < FLOAT32_MIN_POSITIVE
    )
    ratio = subnormal / len(pattern.y)
    assert ratio < 0.05, f"Too many subnormal intensity values: ratio={ratio}"


def test_pattern_new_rejects_mismatched_xy_lengths():
    with pytest.raises(ValueError, match="x and y must have the same length"):
        geddes.Pattern([10.0], [100.0, 101.0])


def test_pattern_new_rejects_non_ascending_x():
    for x in ([20.0, 10.0], [10.0, 10.0]):
        with pytest.raises(ValueError, match="strictly increasing"):
            geddes.Pattern(x, [100.0, 101.0])


def test_pattern_new_rejects_nan_x():
    nan = float("nan")
    for x in ([10.0, nan], [nan], [nan, 20.0]):
        y = [100.0] * len(x)
        with pytest.raises(ValueError, match="strictly increasing"):
            geddes.Pattern(x, y)


def test_read_reverses_descending_xrdml_axis():
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

    pattern = geddes.read_bytes(data, "descending.xrdml")
    assert pattern.x == [10.0, 15.0, 20.0]
    assert pattern.y == [102.0, 101.0, 100.0]
