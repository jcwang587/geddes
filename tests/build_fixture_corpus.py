"""Rebuild committed full-array references without importing Geddes or Rietx.

Development-only: requires numpy and gemmi. The fixture-specific decoders below
use Python's XML/ZIP/struct facilities and gemmi's CIF grammar. Synthetic cases
use literal known arrays. Run with --rietx-source PATH to refresh vendored inputs.
"""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
import struct
import xml.etree.ElementTree as ET
import zipfile

import gemmi
import numpy as np

ROOT = Path(__file__).resolve().parent / "data" / "formats"
REV = "88d2353f446d98632ef233437000cbe7bc4e1142"
SOURCE = f"https://github.com/yue-here/rietx/blob/{REV}/tests/data/"
REAL = ["rigaku_powder.rasx", "rigaku_zno_counts.rasx", "rigaku_nims.ras",
        "panalytical_attenuator.xrdml", "panalytical_mesh.xrdml", "bruker_absorber.brml",
        "11BM_NAC.fxye", "11BM_LaB6_660a.fxye", "11BM_Si640c.xy", "FAP.XRA",
        "nist_srm660c_100a.cif"]
fixtures = []


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def add(name, fmt, path, x, y, *, scan=0, block=None, provenance=None, note=None):
    x, y = np.asarray(x, dtype=float), np.asarray(y, dtype=float)
    if len(x) > 1 and np.all(np.diff(x) < 0):
        x, y = x[::-1], y[::-1]
    assert x.shape == y.shape and x.ndim == 1 and len(x), name
    assert np.all(np.isfinite(x)) and np.all(np.isfinite(y)) and np.all(np.diff(x) > 0), name
    ref = ROOT / "references" / f"{name}.csv"
    np.savetxt(ref, np.column_stack((x, y)), delimiter=",", fmt="%.17g", header="x,y", comments="")
    source = ROOT / path
    fixtures.append(dict(id=name, format=fmt, path=path, reference=f"references/{name}.csv",
                         scan=scan, block=block, points=len(x), rtol=1e-12, atol=1e-9,
                         sha256=digest(source), reference_sha256=digest(ref),
                         provenance=provenance or "Geddes-authored synthetic pattern; MIT",
                         note=note))


def tree(raw):
    root = ET.fromstring(raw)
    for node in root.iter():
        node.tag = node.tag.rsplit("}", 1)[-1]
    return root


def xrdml(path, scan=0):
    root = tree(path.read_bytes())
    p = root.findall("xrdMeasurement/scan")[scan].find("dataPoints")
    intensity = p.find("intensities")
    raw = intensity is None
    if raw:
        intensity = p.find("counts")
    y = np.array([float(v) for v in intensity.text.split()])
    pos = next(n for n in p.findall("positions") if n.get("axis") == "2Theta")
    listed = pos.find("listPositions")
    x = np.array([float(v) for v in listed.text.split()]) if listed is not None else np.linspace(
        float(pos.findtext("startPosition")), float(pos.findtext("endPosition")), len(y))
    attenuation = p.findtext("beamAttenuationFactors")
    if raw and attenuation:
        y *= np.array([float(v) for v in attenuation.split()])
    return x, y


def rasx(path):
    # Every selected real fixture has this declared profile; no reader discovery is used.
    with zipfile.ZipFile(path) as archive:
        lines = archive.read("Data0/Profile0.txt").decode("utf-8-sig").splitlines()
    a = np.array([[float(v) for v in line.split()[:2]] for line in lines if line.strip()])
    return a[:, 0], a[:, 1]


def gsas(path):
    lines = path.read_text().splitlines()
    at = next(i for i, line in enumerate(lines) if line.startswith("BANK"))
    header = lines[at].split()
    if header[-1] == "FXYE":
        a = np.array([[float(v) for v in line.split()[:2]] for line in lines[at+1:] if line.strip()])
        return a[:, 0] / 100, a[:, 1]
    if header[-1] == "ESD":
        vals = [float(line[i:i+8]) for line in lines[at+1:] if line.strip()
                for i in range(0, len(line.rstrip()), 8)]
        y = np.array(vals[::2][:int(header[2])])
    else:
        y = np.array([float(v) for line in lines[at+1:] for v in line.split()][:int(header[2])])
    return float(header[5])/100 + np.arange(len(y))*float(header[6])/100, y


def main():
    args = argparse.ArgumentParser()
    args.add_argument("--rietx-source", type=Path)
    args = args.parse_args()
    (ROOT / "references").mkdir(exist_ok=True)
    if args.rietx_source:
        for name in REAL:
            shutil.copy2(args.rietx_source / "tests" / "data" / name, ROOT / "real" / name)

    x, y = [10, 10.125, 10.25, 10.375, 10.5], [4, 81, 144, 9, 1]
    for suffix, fmt in [("xy", "xy"), ("csv", "csv"), ("chi", "chi"), ("fxye", "gsas"),
                        ("gsas", "gsas"), ("ras", "ras"), ("uxd", "uxd"), ("cif", "pdcif")]:
        add(f"synthetic_{suffix}", fmt, f"text/profile.{suffix}", x, y)
    for fmt in ["ras", "uxd", "cif"]:
        add(f"synthetic_{fmt}_second", "pdcif" if fmt == "cif" else fmt, f"text/profile.{fmt}",
            [20, 20.25, 20.5], [100, 64, 25], scan=0 if fmt == "cif" else 1,
            block="_other" if fmt == "cif" else None)
    add("synthetic_esd", "gsas", "text/profile_esd.gsas", x, y)
    for name, fmt in [("synthetic_raw3", "bruker_raw3"), ("synthetic_raw4_4byte", "bruker_raw4"),
                      ("synthetic_raw4_8byte", "bruker_raw4")]:
        add(name, fmt, f"bruker/{name}.raw", [10, 10.02, 10.04, 10.06, 10.08, 10.10],
            [12, 34.5, 90, 321.25, 87, 5])
    for fmt in ["xrdml", "rasx", "brml"]:
        if (ROOT / "xml" / f"synthetic.{fmt}").exists():
            add(f"synthetic_{fmt}", fmt, f"xml/synthetic.{fmt}", x, y)

    for name in ["rigaku_powder.rasx", "rigaku_zno_counts.rasx"]:
        add(Path(name).stem, "rasx", f"real/{name}", *rasx(ROOT / "real" / name), provenance=SOURCE+name)
    for name in ["panalytical_attenuator.xrdml", "panalytical_mesh.xrdml"]:
        for scan in ([0, 50, 100] if "mesh" in name else [0]):
            add(f"{Path(name).stem}_{scan}", "xrdml", f"real/{name}",
                *xrdml(ROOT / "real" / name, scan), scan=scan, provenance=SOURCE+name)
    name = "bruker_absorber.brml"
    with zipfile.ZipFile(ROOT / "real" / name) as archive:
        root = tree(archive.read("Experiment0/RawData0.xml"))
    route = next(r for r in root.iter("DataRoute") if r.get("RouteFlag") == "Measured")
    a = np.array([[float(v) for v in d.text.split(",")] for d in route.iter("Datum")])
    add("bruker_absorber", "brml", f"real/{name}", a[:, 2], a[:, 7], provenance=SOURCE+name,
        note="Independent fixture-specific columns: 2Theta=2, intensity=7; stored absorber correction retained.")
    name = "rigaku_nims.ras"
    text = (ROOT / "real" / name).read_text()
    body = text.split("*RAS_INT_START", 1)[1].split("*RAS_INT_END", 1)[0]
    a = np.array([[float(v) for v in line.split()[:2]] for line in body.splitlines() if line.strip()])
    add("rigaku_nims", "ras", f"real/{name}", a[:, 0], a[:, 1], provenance=SOURCE+name)
    for name in ["11BM_NAC.fxye", "11BM_LaB6_660a.fxye", "FAP.XRA"]:
        add(Path(name).stem, "gsas", f"real/{name}", *gsas(ROOT / "real" / name), provenance=SOURCE+name)
    name = "11BM_Si640c.xy"
    a = np.loadtxt(ROOT / "real" / name)
    add("11BM_Si640c", "xy", f"real/{name}", a[:, 0], a[:, 1], provenance=SOURCE+name)
    name = "nist_srm660c_100a.cif"
    doc = gemmi.cif.read(str(ROOT / "real" / name))
    b = next(b for b in doc if b.name.endswith("_meas"))
    x = [gemmi.cif.as_number(v) for v in b.find_loop("_pd_proc_2theta_corrected")]
    y = [gemmi.cif.as_number(v) for v in b.find_loop("_pd_proc_intensity_total")]
    add("nist_srm660c", "pdcif", f"real/{name}", x, y, block="_meas", provenance=SOURCE+name)

    add("existing_xrdml", "xrdml", "../xrdml/sample.xrdml", *xrdml(ROOT / "../xrdml/sample.xrdml"),
        provenance="Existing Geddes fixture; same measurement as Rietx panalytical_powder.xrdml")
    add("existing_rasx", "rasx", "../rasx/sample.rasx", *rasx(ROOT / "../rasx/sample.rasx"),
        provenance="Existing Geddes fixture")
    add("existing_gsas", "gsas", "../gsas_raw/gsas.raw", *gsas(ROOT / "../gsas_raw/gsas.raw"),
        provenance="Existing Geddes fixture")
    for fmt in ["xy", "csv"]:
        a = np.loadtxt(ROOT / ".." / fmt / f"sample.{fmt}")
        add(f"existing_{fmt}", fmt, f"../{fmt}/sample.{fmt}", a[:, 0], a[:, 1], provenance="Existing Geddes fixture")
    for name, offset, points, stride, start, step in [
        ("bruker4_v5converter", 8117, 4059, 4, 37.0001, 0.020454544980000003),
        ("bruker4_diffrac_eva", 1311, 7134, 8, 10, 0.010520299896597862)]:
        path = f"../bruker_raw/{name}.raw"
        raw = (ROOT / path).read_bytes()
        y = [struct.unpack_from("<f", raw, offset+i*stride)[0] for i in range(points)]
        add(name, "bruker_raw4", path, start+np.arange(points)*step, y,
            provenance="Existing Geddes fixture; literal offsets independently verified against file layout",
            note="EVA intensities scrambled: verifies byte decoding, not physical measurement correctness." if "eva" in name else None)
    payload = dict(schema_version=1, rietx_reference_commit=REV,
                   reference_method="Known synthetic arrays; fixture-specific independent Python/NumPy/XML/ZIP/struct/gemmi extraction. No Geddes or Rietx reader used.", fixtures=fixtures)
    (ROOT / "manifest.json").write_text(json.dumps(payload, indent=2)+"\n")
    print(f"Wrote {len(fixtures)} cases, {sum(f['points'] for f in fixtures):,} checked points")
    for f in fixtures:
        print(f["id"], f["points"])


if __name__ == "__main__":
    main()
