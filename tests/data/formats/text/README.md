# Synthetic text profile fixtures

These files were authored for Geddes, are covered by the repository's MIT
license, and contain no measured or third-party experimental data. They encode
the same independently specified five-point pattern:

- 2theta (degrees): `[10, 10.125, 10.25, 10.375, 10.5]`
- intensity (stored units): `[4, 81, 144, 9, 1]`

The RAS, UXD, and CIF fixtures additionally contain a selectable second pattern
with 2theta `[20, 20.25, 20.5]` and intensity `[100, 64, 25]`. The RAS file stores
that second scan in descending order to verify paired reversal. UXD has a third
counts-only scan with inherited start and increment metadata.

`profile.gsas` stores counts-only STD records; `profile_esd.gsas` stores fixed
eight-character intensity/esd pairs. The FXYE and ESD fixtures deliberately
include a zero uncertainty; the CIF similarly
includes a zero weight. Geddes preserves those x/y points because it does not
perform uncertainty processing. The RAS attenuator column is also ignored.

Format facts were checked against the Rietx format modules (retrieved 2026-09-11)
and their documented specifications:

- [GSAS](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/gsas.py):
  CONS/CONST centidegree axis; STD, ESD, and FXYE record layouts.
- [RAS](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/ras.py):
  header and intensity block markers, scan-axis key, stored intensity column.
- [UXD](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/uxd.py):
  explicit positions or `_START`/`_STEPSIZE`, counts or cps, inherited headers.
- [CHI](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/chi.py):
  four-line header, including a point count which must not become a data row.
- [pdCIF](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/pdcif.py):
  measured-profile tag alternatives and block selection.

`tests/text_formats.rs` also constructs small malformed and boundary examples
directly, including fused full-width ESD fields and CIF quoted/multiline values.
