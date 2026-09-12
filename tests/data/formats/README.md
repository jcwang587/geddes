# XRD reader fixture corpus

[manifest.json](manifest.json) is the shared correctness and benchmark index.
Each entry gives a file path relative to this directory, its selected scan or
CIF block, the expected point count, a full two-column reference CSV, numeric
tolerances, SHA-256 hashes and provenance. Multiple entries can select different
patterns from the same file.

The current manifest contains **38 cases and 312,202 checked points**, covering
all ten supported reader families across synthetic and experimental inputs.

## Experimental inputs

Files in `real/` were copied from the
[Rietx test-data directory](https://github.com/yue-here/rietx/tree/88d2353f446d98632ef233437000cbe7bc4e1142/tests/data)
at commit **`88d2353f446d98632ef233437000cbe7bc4e1142`**. Original sources and
license descriptions below follow its
[provenance record](https://github.com/yue-here/rietx/blob/88d2353f446d98632ef233437000cbe7bc4e1142/tests/data/README.md).
Their numerical contents are retained, including zero-uncertainty points.

| File | Profile used | Original source | Upstream terms |
|---|---|---|---|
| `11BM_NAC.fxye` | 59,498 explicit FXYE points, NAC powder | [GSAS-II tutorials](https://github.com/AdvancedPhotonSource/GSAS-II-tutorials), `TOF-CW Joint Refinement/data/` | Publicly distributed Argonne/APS tutorial data; upstream describes U.S. Government work |
| `11BM_LaB6_660a.fxye` | 132,992 FXYE points, LaB6 SRM 660a | [GSAS-II tutorials](https://github.com/AdvancedPhotonSource/GSAS-II-tutorials), `FitPeaks/data/11bmb_3844.fxye` | Same Argonne/APS status |
| `11BM_Si640c.xy` | 48,000 x/y points, silicon SRM 640c | Published APS 11-BM Standards Data, recovered by Rietx from the Internet Archive | Public Argonne/APS standard-reference measurement; upstream describes U.S. Government work |
| `FAP.XRA` | 5,753 STD channels, fluorapatite | [GSAS-II tutorials](https://github.com/AdvancedPhotonSource/GSAS-II-tutorials), `LabData/data/` | Same Argonne/APS status |
| `nist_srm660c_100a.cif` | 5,332 points from the `_meas` profile block | [NIST data record mds2-2315](https://data.nist.gov/od/id/mds2-2315) | NIST open data; upstream describes U.S. Government work |
| `rigaku_nims.ras` | 3,501 points, 25–60 degrees | [nims-mdpf/M-DaC_XRD](https://github.com/nims-mdpf/M-DaC_XRD), `source/XRD_RIGAKU.ras` | Original project MIT license |
| `rigaku_powder.rasx` | 2,726 points | [FAIRmat readers-xrd](https://github.com/FAIRmat-NFDI/readers-xrd), `tests/data/TwoTheta_scan_powder.rasx` | Apache-2.0 |
| `rigaku_zno_counts.rasx` | 7,001 points | [FAIRmat readers-xrd](https://github.com/FAIRmat-NFDI/readers-xrd), `tests/data/ZnO-ALD-training_001_1_0-000_0-000.rasx` | Apache-2.0 |
| `panalytical_attenuator.xrdml` | 1,800 points, raw counts and varying attenuation factors | [FAIRmat readers-xrd](https://github.com/FAIRmat-NFDI/readers-xrd), `tests/data/m54313_om2th_10.xrdml` | Apache-2.0 |
| `panalytical_mesh.xrdml` | 101 available scans of 255 explicit positions; selected scans listed in manifest | [FAIRmat readers-xrd](https://github.com/FAIRmat-NFDI/readers-xrd), `tests/data/m82762_rc1mm_1_16dg_src_slit_phi-101_3dg_-420_mesh_long.xrdml` | Apache-2.0 |
| `bruker_absorber.brml` | 2,001 points with intensity in the declared intensity field | [FAIRmat readers-xrd](https://github.com/FAIRmat-NFDI/readers-xrd), `tests/data/23-012-AG_2thomegascan_long.brml` | Apache-2.0 |

The NAC file's actual 59,498 complete rows differ from both its BANK header
(59,497) and Rietx's older provenance prose (54,000). Geddes keeps every explicit
FXYE x/y row. Its full reference therefore has **59,498 points**. Rietx's
uncertainty filtering can produce another count; that does not change the
contents of the input file.

The [Apache-2.0 license](licenses/LICENSE) accompanies FAIRmat-derived files.
[Rietx's MIT notice](licenses/rietx-MIT.txt) is retained separately; it does not
replace the original data sources' terms. The RAS input's original MIT source is
the NIMS project linked above; its [MIT notice](licenses/nims-MIT.txt), including
the original copyright statement, is included.

## Synthetic and existing Geddes inputs

[text/](text/README.md) contains authored examples for ASCII, GSAS STD/ESD/FXYE,
RAS, UXD, CHI and pdCIF. Their intended x/y arrays are explicitly documented,
including selectable secondary profiles. [bruker/](bruker/README.md) contains
authored RAW3 and RAW4 binary examples with independently specified values.
XML/ZIP examples, when listed in the manifest, likewise encode known arrays.
These authored examples use Geddes's MIT license and are not experimental data.

The manifest also points to the original Geddes samples elsewhere in
`tests/data/`; their provenance is marked separately. The existing
`bruker4_diffrac_eva.raw` is byte-identical to Rietx's
`bruker_raw4_scrambled.raw`. Its scrambled intensities support byte-decoding and
performance tests, not physical-profile validation. The existing XRDML sample
contains the same measurement as Rietx's `panalytical_powder.xrdml`.

## Independent references

[../../build_fixture_corpus.py](../../build_fixture_corpus.py) regenerates the
full x/y CSV references using literal synthetic arrays and independent,
fixture-specific Python/NumPy/XML/ZIP/`struct`/gemmi extraction. It never calls a
Geddes or Rietx reader. Reference intensities follow Geddes's defined contract:
native units, XRDML raw-count attenuation correction, no extra correction to
BRML or processed XRDML intensities, and no uncertainty-based row filtering.

See the [test guide](../../README.md) for regeneration commands and the sibling
[geddes-test benchmark](../../../../geddes-test/README.md) for timing methodology.
The benchmark validates full arrays before timing and excludes numerical or
selection mismatches from speed comparisons.
