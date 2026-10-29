# /// script
# requires-python = ">=3.10"
# dependencies = ["tomlkit==0.13.3"]
# ///
"""Bump the shared release version and synchronize Cargo.lock."""

import re
import subprocess
from pathlib import Path

import tomlkit


def bump_version(root: Path) -> str:
    manifest = root / "Cargo.toml"
    library = root / "pyo3-stub-gen/Cargo.toml"
    lockfile = root / "Cargo.lock"
    originals = {path: path.read_bytes() for path in (manifest, library, lockfile)}
    workspace = tomlkit.parse(originals[manifest].decode())
    package = tomlkit.parse(originals[library].decode())

    old_version = workspace["workspace"]["package"]["version"]
    if not re.fullmatch(
        r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", old_version
    ):
        raise ValueError(
            f"Expected a stable major.minor.patch version, got {old_version!r}"
        )
    major, minor, patch = map(int, old_version.split("."))
    new_version = f"{major}.{minor}.{patch + 1}"

    derive = package["dependencies"]["pyo3-stub-gen-derive"]
    if derive["version"] != old_version:
        raise ValueError(
            "The derive dependency version must match workspace.package.version"
        )
    workspace["workspace"]["package"]["version"] = new_version
    derive["version"] = new_version

    try:
        manifest.write_text(tomlkit.dumps(workspace), encoding="utf-8")
        library.write_text(tomlkit.dumps(package), encoding="utf-8")
        # Keep external dependencies at their locked versions.
        subprocess.run(["cargo", "update", "--workspace"], cwd=root, check=True)
    except Exception:
        for path, content in originals.items():
            path.write_bytes(content)
        raise
    return new_version


if __name__ == "__main__":
    print(bump_version(Path(__file__).resolve().parents[1]))
