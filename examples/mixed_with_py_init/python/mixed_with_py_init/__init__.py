"""mixed_with_py_init - Example demonstrating __init__.py and __init__.pyi coexistence problem.

This __init__.py re-exports:
- SomeClass from .submodule (pure Python)
- NativeClass from ._native (Rust via PyO3)

When pyo3-stub-gen generates __init__.pyi, it only contains Rust types.
Type checkers (mypy, pyright, pyrefly) prioritize .pyi over .py,
so the SomeClass re-export becomes invisible to type checking.

Expected behavior: Both SomeClass and NativeClass should be available
from `mixed_with_py_init` module.

Actual behavior with generated __init__.pyi: Only NativeClass is visible
to type checkers.
"""

# Re-export from pure Python submodule
from mixed_with_py_init.submodule import SomeClass, some_function

# Re-export from Rust native module
from mixed_with_py_init._native import NativeClass, native_function

# DirectClass is defined with module="mixed_with_py_init" in Rust,
# but at runtime it's actually in _native
from mixed_with_py_init._native import DirectClass

__all__ = [
    "SomeClass",
    "some_function",
    "NativeClass",
    "native_function",
    "DirectClass",
]
