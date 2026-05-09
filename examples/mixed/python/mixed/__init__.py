"""mixed - Example demonstrating mixed Python/Rust module layout.

This __init__.py re-exports:
- SomeClass, some_function from .submodule (pure Python)
- A, greet_main from .main_mod (Rust via PyO3)

This demonstrates that pyo3-stub-gen correctly:
1. Does NOT generate __init__.pyi for this module (would shadow this file)
2. Only generates stubs for PyO3-generated modules (main_mod and below)
"""

# Re-export from pure Python submodule
from mixed.submodule import SomeClass, some_function

# Re-export from Rust native module
from mixed.main_mod import A, greet_main

__all__ = [
    "SomeClass",
    "some_function",
    "A",
    "greet_main",
]
