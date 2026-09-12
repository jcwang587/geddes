# Supported formats

Geddes loads one profile at a time. Use `scan` for a zero-based scan, range, or
bank index and `block` for powder CIF selection; the default is the first
pattern. See [usage](reading-patterns.md#scan-and-block-selection) for examples.

| Format | Common extensions | Supported variants |
|---|---|---|
| ASCII columns | `.xy`, `.xye`, `.csv`, `.dat`, `.prn`, `.txt` | Whitespace or comma-separated x/y; extra columns ignored |
| GSAS | `.gsas`, `.gsa`, `.fxye`, `.gda`, `.xra`, `.raw` | `CONS`/`CONST` with STD, ESD, or FXYE records; centidegrees converted to degrees |
| Bruker RAW | `.raw` | RAW3 and RAW4 2theta scans |
| Rigaku RAS | `.ras` | Text diffraction profiles |
| Rigaku RASX | `.rasx` | Diffraction profiles, selected in the file's scan order |
| Bruker/Siemens UXD | `.uxd` | `_COUNTS`, `_CPS`, `_2THETACOUNTS`, `_2THETACPS`; explicit positions or start/step values |
| Bruker BRML | `.brml` | Integrated 1D profiles; uniquely identified measured data or the sole dataset within the selected scan |
| PANalytical XRDML | `.xrdml` | Counts or intensities with a 2theta range or position list |
| FIT2D/pyFAI CHI | `.chi` | Four-line header and x/y rows matching the declared point count |
| Powder CIF | `.cif` | Measured powder profiles; corrected 2theta and processed total intensity take precedence when present |

Geddes checks content before using the filename as a hint, distinguishing GSAS
text from Bruker binary files that both use `.raw`. Text inputs support UTF-8
with or without a byte-order mark and UTF-16 with one. ASCII files can include
common comment lines and prose headers.

<a id="limits"></a>

Positions must represent 2theta in degrees. Geddes does not infer a wavelength
or convert Q, d-spacing, or time-of-flight axes. A successful read cannot verify
units absent from the file. Unsupported inputs include RAW1/RAW2, detector
frames, DIF peak lists, structural CIFs without a profile, and non-2theta scans.
GSAS `RALF`, `SLOG`, `TIME_MAP`, `COND`, and `CONQ` binnings, ALT/FXY records,
and compressed STD records are unsupported.

Intensities retain their stored units, including cps. Geddes multiplies XRDML
raw counts by supplied attenuation factors; reported XRDML intensities and BRML
values remain unchanged. See [intensity handling](reading-patterns.md#intensity-values)
for Rigaku attenuators and other conventions.
