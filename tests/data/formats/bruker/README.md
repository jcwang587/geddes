# Bruker format fixtures

These three small synthetic files encode the complete pattern in `expected.xy`:
RAW3 (`RAW1.01`), RAW4 with a four-byte intensity record, and RAW4 with an
eight-byte record whose final four bytes are the integer 1. Intensities are
invented, not a physical measurement. They establish numeric decoding and
prevent regression to reading record flags as intensities.

The files were packed from the public field descriptions in
[Rietx's RAW format notes](https://github.com/yue-here/rietx/blob/main/src/rietx/io/formats/bruker_raw.py)
and [independent binary writers](https://github.com/yue-here/rietx/blob/main/tests/writers_xrd.py),
using literal offsets independently of Geddes's reader. The integration tests
also pack malformed and multiple-scan examples from literal field offsets.

These synthetic fixtures cannot independently establish the vendor's format.
The existing `tests/data/bruker_raw/bruker4_v5converter.raw` and
`bruker4_diffrac_eva.raw` exercise real RAW4 structures and distinct record
strides. The EVA intensities were scrambled upstream and are useful only for
checking bytes are decoded faithfully, not for physical intensity validation.
No vendor-produced RAW3 sample is included here.
