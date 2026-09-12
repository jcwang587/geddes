"""Release gate regressions; run separately with Python 3.12 unittest."""

from contextlib import redirect_stderr, redirect_stdout
import io
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

if sys.version_info < (3, 11):
    raise unittest.SkipTest("Release validation runs under Python 3.12 in CI")

import check_release_version as gate


class ReleaseVersionTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        for filename in ("Cargo.toml", "Cargo.lock", "node/Cargo.toml", "node/Cargo.lock", "node/package.json", "node/package-lock.json", "pyproject.toml"):
            target = self.root / filename
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(gate.ROOT / filename, target)
        self.version = gate.check_versions(self.root)

    def test_all_manifests_agree(self):
        self.assertEqual(gate.check_versions(self.root, f"v{self.version}", f"v{self.version}"), self.version)

    def test_requested_version_must_match(self):
        with self.assertRaisesRegex(ValueError, "Requested version"):
            gate.check_versions(self.root, requested="999.999.999")

    def test_manual_tag_must_match(self):
        with self.assertRaisesRegex(ValueError, "Tag"):
            gate.check_versions(self.root, tag="v999.999.999")

    def test_stale_node_package_is_refused(self):
        path = self.root / "node/package.json"
        data = json.loads(path.read_text())
        data["version"] = "999.999.999"
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "node/package.json"):
            gate.check_versions(self.root)

    def test_stale_lockfile_is_refused(self):
        path = self.root / "node/package-lock.json"
        data = json.loads(path.read_text())
        data["packages"][""]["version"] = "999.999.999"
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "packages"):
            gate.check_versions(self.root)

    def test_pr_ref_is_not_interpreted_as_a_tag(self):
        with patch.object(gate, "ROOT", self.root), patch("sys.argv", ["check_release_version.py"]), patch.dict(os.environ, {"GITHUB_REF_TYPE": "branch", "GITHUB_REF_NAME": "123/merge", "GITHUB_OUTPUT": ""}), redirect_stdout(io.StringIO()):
            self.assertEqual(gate.main(), 0)

    def test_direct_tag_run_cannot_bypass_version_validation(self):
        with patch.object(gate, "ROOT", self.root), patch("sys.argv", ["check_release_version.py"]), patch.dict(os.environ, {"GITHUB_REF_TYPE": "tag", "GITHUB_REF_NAME": "v999.999.999", "GITHUB_OUTPUT": ""}), redirect_stderr(io.StringIO()):
            self.assertEqual(gate.main(), 1)


if __name__ == "__main__":
    unittest.main()
