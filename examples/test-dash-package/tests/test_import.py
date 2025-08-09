"""Test that the module can be imported correctly despite having a dash in the package name."""

import test_dash_package
import pathlib


def test_test_function():
    assert test_dash_package.test_function() == 42


def test_stub_file_naming():
    """Test that the stub file was generated with underscores instead of dashes."""
    # The stub file should have underscores, not dashes
    stub_file = pathlib.Path(__file__).parent.parent / "test_dash_package.pyi"
    assert stub_file.exists(), "Stub file with underscores should exist"

    # The old dash version should not exist
    dash_stub_file = pathlib.Path(__file__).parent.parent / "test-dash-package.pyi"
    assert not dash_stub_file.exists(), "Stub file with dashes should not exist"
