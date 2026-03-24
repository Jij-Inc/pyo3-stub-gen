"""Example of user-managed re-exports.

The PyO3 shared library is at `empty_super_module.main_mod` and its submodules
(sub_mod, deep) are accessible as `main_mod.sub_mod`, `main_mod.deep`, etc.

This __init__.py re-exports them at the top level so users can access them as
`empty_super_module.sub_mod` instead of `empty_super_module.main_mod.sub_mod`.

This re-export is the USER's responsibility - pyo3-stub-gen only generates stubs
at the actual module paths (main_mod/sub_mod/__init__.pyi, etc.).
"""

# ruff: noqa: F403
from .main_mod import sub_mod, deep

__all__ = ["sub_mod", "deep"]
