# /// script
# requires-python = ">=3.10"
# dependencies = ["tomlkit==0.13.3"]
# ///
"""Exercise release version updates in a temporary Cargo workspace."""

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import tomlkit

from bump_version import bump_version


class BumpVersionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.env = patch.dict(os.environ, {"CARGO_NET_OFFLINE": "true"})
        self.env.start()
        self.addCleanup(self.env.stop)
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nresolver = "2"\nmembers = ["pyo3-stub-gen", '
            '"pyo3-stub-gen-derive", "example", "independent"]\n\n'
            '[workspace.package]\nversion = "0.23.9" # release version\n',
            encoding="utf-8",
        )
        for name in ("pyo3-stub-gen", "pyo3-stub-gen-derive", "example", "independent"):
            directory = self.root / name
            (directory / "src").mkdir(parents=True)
            (directory / "src/lib.rs").touch()
            version = (
                'version = "0.22.6"'
                if name == "independent"
                else "version.workspace = true"
            )
            content = f'[package]\nname = "{name}"\n{version}\nedition = "2021"\n'
            if name == "pyo3-stub-gen":
                content += (
                    "\n[dependencies.pyo3-stub-gen-derive]\n"
                    'path = "../pyo3-stub-gen-derive"\n'
                    'version = "0.23.9" # keep in sync\n'
                )
            (directory / "Cargo.toml").write_text(content, encoding="utf-8")
        subprocess.run(
            ["cargo", "generate-lockfile"],
            cwd=self.root,
            check=True,
            capture_output=True,
        )
        self.originals = {
            path: path.read_bytes()
            for path in (*self.root.rglob("Cargo.toml"), self.root / "Cargo.lock")
        }

    def assert_unchanged(self):
        for path, content in self.originals.items():
            self.assertEqual(path.read_bytes(), content, str(path))

    def test_bump_preserves_formatting_and_updates_inherited_lock_versions(self):
        self.assertEqual(bump_version(self.root), "0.23.10")
        for path, content in self.originals.items():
            if path.name == "Cargo.toml":
                self.assertEqual(
                    path.read_bytes(), content.replace(b'"0.23.9"', b'"0.23.10"')
                )
        lock = tomlkit.parse((self.root / "Cargo.lock").read_text(encoding="utf-8"))
        expected = {
            "pyo3-stub-gen": "0.23.10",
            "pyo3-stub-gen-derive": "0.23.10",
            "example": "0.23.10",
            "independent": "0.22.6",
        }
        self.assertEqual({p["name"]: p["version"] for p in lock["package"]}, expected)
        metadata = subprocess.run(
            ["cargo", "metadata", "--locked", "--offline", "--format-version=1"],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(
            {p["name"]: p["version"] for p in json.loads(metadata.stdout)["packages"]},
            expected,
        )

    def test_rejects_prerelease_before_writing(self):
        manifest = self.root / "Cargo.toml"
        manifest.write_bytes(
            self.originals[manifest].replace(b'"0.23.9"', b'"0.23.9-rc.1"')
        )
        self.originals[manifest] = manifest.read_bytes()
        with self.assertRaisesRegex(ValueError, "stable major.minor.patch"):
            bump_version(self.root)
        self.assert_unchanged()

    def test_rejects_mismatched_dependency_before_writing(self):
        manifest = self.root / "pyo3-stub-gen/Cargo.toml"
        manifest.write_bytes(self.originals[manifest].replace(b'"0.23.9"', b'"0.23.8"'))
        self.originals[manifest] = manifest.read_bytes()
        with self.assertRaisesRegex(ValueError, "derive dependency version must match"):
            bump_version(self.root)
        self.assert_unchanged()

    def test_cargo_failure_restores_manifests_and_lockfile(self):
        def fail(*args, **kwargs):
            (self.root / "Cargo.lock").write_text("partial update", encoding="utf-8")
            raise subprocess.CalledProcessError(1, args[0])

        with patch("bump_version.subprocess.run", side_effect=fail):
            with self.assertRaises(subprocess.CalledProcessError):
                bump_version(self.root)
        self.assert_unchanged()


if __name__ == "__main__":
    unittest.main()
