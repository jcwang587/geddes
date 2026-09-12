# XML container samples

`synthetic.xrdml`, `synthetic.rasx`, and `synthetic.brml` encode the entire
five-point pattern in `expected.xy`: 2theta 10–10.5° at 0.125° intervals,
intensities 4, 81, 144, 9, 1. These are invented values, not measured data.

The XML is written directly from each self-describing container's documented
structure: XRDML `positions`/`counts`, RASX manifest-listed profile/conditions,
and BRML manifest-listed data with named `RawDataView` columns. The BRML
intensity is column 1 and TwoTheta is column 4, intentionally avoiding a
fixed-position assumption. ZIP members use uncompressed storage.

Format references: Rietx's
[XRDML](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/xrdml.py),
[RASX](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/rasx.py), and
[BRML](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/brml.py)
field descriptions. These files contain no copied measurement or reader code.
