"""Check package version agreement, and a release/tag version when supplied.

Ordinary branch and pull-request checks need no arguments. GitHub tag runs are
validated automatically; release preparation passes ``--version 1.2.3``.
This script reads manifests only and never updates package versions.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?\Z")


def check_versions(root: Path, requested: str | None = None, tag: str | None = None) -> str:
    def toml(name):
        return tomllib.loads((root / name).read_text(encoding="utf-8"))

    version = toml("Cargo.toml")["package"]["version"]
    if not isinstance(version, str) or not VERSION.fullmatch(version):
        raise ValueError(f"Cargo.toml has an invalid release version: {version!r}")
    versions = {"node/Cargo.toml": toml("node/Cargo.toml")["package"]["version"]}
    package = json.loads((root / "node/package.json").read_text(encoding="utf-8"))
    lock = json.loads((root / "node/package-lock.json").read_text(encoding="utf-8"))
    versions["node/package.json"] = package["version"]
    versions["node/package-lock.json"] = lock["version"]
    versions["node/package-lock.json packages['']"] = lock["packages"][""]["version"]
    for filename, names in (("Cargo.lock", ("geddes",)), ("node/Cargo.lock", ("geddes", "geddes-node"))):
        packages = toml(filename)["package"]
        for name in names:
            matches = [p["version"] for p in packages if p["name"] == name and "source" not in p]
            if len(matches) != 1:
                raise ValueError(f"{filename} must contain exactly one local {name} package")
            versions[f"{filename} ({name})"] = matches[0]
    project = toml("pyproject.toml")["project"]
    if "version" not in project.get("dynamic", []) or "version" in project:
        raise ValueError("pyproject.toml must derive its version dynamically from Cargo.toml")
    for filename, actual in versions.items():
        if actual != version:
            raise ValueError(f"{filename} version {actual!r} does not match Cargo.toml {version!r}")
    if requested is not None:
        requested = requested.removeprefix("v")
        if requested != version:
            raise ValueError(f"Requested version {requested!r} does not match Cargo.toml {version!r}")
    if tag is not None and tag != f"v{version}":
        raise ValueError(f"Tag {tag!r} does not match package version v{version}")
    return version


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", help="Requested release version, optionally prefixed with v")
    parser.add_argument("--tag", help="Expected tag; defaults to GITHUB_REF_NAME on tag runs")
    args = parser.parse_args()
    tag = args.tag
    if tag is None and os.environ.get("GITHUB_REF_TYPE") == "tag":
        tag = os.environ.get("GITHUB_REF_NAME", "")
    try:
        version = check_versions(ROOT, requested=args.version, tag=tag)
    except (ValueError, KeyError, OSError) as exc:
        print(f"Version check failed: {exc}", file=sys.stderr)
        return 1
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as stream:
            stream.write(f"version={version}\ntag=v{version}\n")
    print(f"Package versions agree: {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
