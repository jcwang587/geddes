# Supported formats

Geddes loads one-dimensional XRD profiles with a 2theta axis. The table describes
supported layouts; an extension alone does not establish that a file contains
one of those layouts.

| Format | Common extensions | Supported layouts | Selection |
|---|---|---|---|
| ASCII columns | `.xy`, `.xye`, `.csv`, `.dat`, `.prn`, `.txt` | Whitespace or comma-separated x/y, with extra columns ignored | One pattern |
| GSAS | `.gsas`, `.gsa`, `.fxye`, `.gda`, `.xra`, `.raw` | `CONS`/`CONST` with STD, ESD, or FXYE records | Bank index |
| Bruker RAW | `.raw` | Binary RAW3 and RAW4, including supported four- and eight-byte RAW4 point records | Scan index |
| Rigaku RAS | `.ras` | Text header and intensity blocks | Scan index |
| Rigaku RASX | `.rasx` | ZIP profile data with associated scan-axis declarations | Scan index |
| Bruker/Siemens UXD | `.uxd` | Counts or cps with explicit positions or start/step values | Range index |
| Bruker BRML | `.brml` | ZIP/XML measured 1D routes with declared position and intensity fields | Scan index |
| PANalytical XRDML | `.xrdml` | Counts or intensities with 2theta start/end values or explicit position lists | Scan index |
| FIT2D/pyFAI CHI | `.chi` | Four-line header followed by x/y rows | One pattern |
| Powder CIF | `.cif` | Measured-profile loops or a scalar angular range with an intensity loop | Block name |

See [reading patterns](reading-patterns.md) for selection syntax.

## Detection and text encoding

Geddes examines RAW magic bytes, ZIP members, XML tags, and text markers before
using the filename as a hint. This distinguishes, for example, a GSAS text
`.raw` file from a Bruker binary `.raw` file. Plain numeric text also has an ASCII
fallback.

Text readers accept UTF-8 and UTF-16 byte-order marks. Legacy encoded comments
can be decoded without changing ASCII numeric fields. ASCII-column readers skip
common comment lines and prose headers, and read the first two numeric columns.

## Format details

### GSAS

`CONS` and `CONST` describe an angular grid in centidegrees; Geddes returns
degrees. STD records contain counts, ESD records contain fixed-width
intensity/uncertainty pairs, and FXYE records contain explicit
position/intensity/uncertainty triples. Only x and y are returned.

STD and ESD use the declared channel count and discard final zero padding.
FXYE retains complete explicit rows, including the extra endpoint written by
some APS exports. Zero uncertainty does not remove a point.

### RAS, RASX, and UXD

RAS and RASX use the stored position and intensity columns. Rigaku's additional
attenuator column is ignored.

UXD accepts `_COUNTS`, `_CPS`, `_2THETACOUNTS`, and `_2THETACPS`. The first two
derive positions from `_START` and `_STEPSIZE`; the others carry explicit
positions. Header values can persist across ranges. The `_DRIVE` declaration is
checked independently of the block marker.

### XRDML and BRML

XRDML reads `<counts>` or `<intensities>` and requires usable 2theta positions.
BRML locates the measured profile's position and intensity fields from its
descriptions instead of assuming fixed column numbers. Their intensity
conventions are described in [reading patterns](reading-patterns.md#intensity-values).

### CHI and powder CIF

CHI skips exactly four header lines and checks the declared point count. Its
axis label is checked for recognized non-2theta coordinates, such as Q or
d-spacing.

Powder CIF supports measured-profile tag alternatives, quoted values, multiline
text, comments, and numeric uncertainties such as `10.125(2)`. Corrected 2theta
and processed total-intensity tags take precedence when available. A structural
CIF without a measured powder profile is not a pattern input.

## Limits

The following data are outside the supported layouts:

- Bruker RAW1 and RAW2, raw detector frames, and unsupported binary record layouts.
- GSAS non-angle binnings, including `RALF`, `SLOG`, `TIME_MAP`, `COND`, and
  `CONQ`; ALT/FXY records and compressed STD records.
- DIF reflection peak lists and structural CIFs without a measured profile.
- Non-2theta scans rejected by a format's axis checks.

Geddes does not infer a wavelength or convert Q, d-spacing, or time-of-flight
axes. Raw detector frames need integration to a 1D 2theta profile first. For
plain ASCII and some text formats with absent or unrecognized axis labels, the
caller must supply data already expressed as 2theta in degrees.
