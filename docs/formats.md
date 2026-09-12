# Supported formats

Geddes loads one profile at a time. Use `index` for a scan, range, or bank's
zero-based position and `block` for powder CIF selection; the default is the
first pattern. See [usage](reading-patterns.md#pattern-selection) for examples.

| Format | Common extensions | Selection | Supported variants |
|---|---|---|---|
| Text | `.xy`, `.xye`, `.csv`, `.dat`, `.prn`, `.txt` | Single pattern | Whitespace or comma-separated x/y; extra columns ignored |
| GSAS | `.gsas`, `.gsa`, `.fxye`, `.gda`, `.xra`, `.raw` | Bank (`index`) | `CONS`/`CONST` with STD, ESD, or FXYE records; centidegrees converted to degrees |
| Bruker RAW | `.raw` | Scan (`index`) | Binary 2theta scans |
| RAS | `.ras` | Scan (`index`) | Text diffraction profiles |
| RASX | `.rasx` | Scan (`index`) | Diffraction profiles, selected in the file's scan order |
| UXD | `.uxd` | Scan (`index`) | `_COUNTS`, `_CPS`, `_2THETACOUNTS`, `_2THETACPS`; explicit positions or start/step values |
| BRML | `.brml` | Scan (`index`) | Integrated 1D profiles; uniquely identified measured data or the sole dataset within the selected scan |
| XRDML | `.xrdml` | Scan (`index`) | Counts or intensities with a 2theta range or position list |
| CHI | `.chi` | Single pattern | Four-line header and x/y rows matching the declared point count |
| Powder CIF | `.cif` | Data block (`block`) | Measured powder profiles; corrected 2theta and processed total intensity take precedence when present |

Geddes checks content before using the filename as a hint, distinguishing GSAS
text from Bruker binary files that both use `.raw`. Text inputs support UTF-8
with or without a byte-order mark and UTF-16 with one. ASCII files can include
common comment lines and prose headers.

<a id="limits"></a>

Bruker RAW support is limited to versions 3 and 4.

Positions must represent 2theta in degrees. Geddes does not infer a wavelength
or convert Q, d-spacing, or time-of-flight axes. A successful read cannot verify
units absent from the file. Unsupported inputs include detector frames, DIF
peak lists, structural CIFs without a profile, and non-2theta scans.
GSAS `RALF`, `SLOG`, `TIME_MAP`, `COND`, and `CONQ` binnings, ALT/FXY records,
and compressed STD records are unsupported.

Intensities retain their stored units, including cps. Geddes multiplies XRDML
raw counts by supplied attenuation factors; reported XRDML intensities and BRML
values remain unchanged. See [intensity handling](reading-patterns.md#intensity-values)
for Rigaku attenuators and other conventions.
