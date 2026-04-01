"""Tests for runtime type alias (define_type_alias!) support.

This module tests that type aliases defined with define_type_alias! macro
can be imported from the generated __init__.py file.
"""

import types


def test_import_type_alias_from_init():
    """Test that AorB type alias can be imported from generate_init_py."""
    # This import would fail if AorB was only a stub-only type alias
    from generate_init_py import AorB

    # Verify it's a union type (types.UnionType in Python 3.10+)
    assert isinstance(AorB, types.UnionType), f"Expected UnionType, got {type(AorB)}"


def test_type_alias_contains_correct_types():
    """Test that AorB union contains the correct types (A and B)."""
    from generate_init_py import A, AorB, B

    # Get the union's __args__ to check the component types
    args = AorB.__args__
    assert len(args) == 2, f"Expected 2 types in union, got {len(args)}"
    assert A in args, f"Expected A in union args, got {args}"
    assert B in args, f"Expected B in union args, got {args}"


def test_type_alias_from_core_module():
    """Test that AorB can also be imported directly from _core module."""
    from generate_init_py._core import AorB

    assert isinstance(AorB, types.UnionType)
