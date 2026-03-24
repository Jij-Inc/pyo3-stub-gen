"""Test file demonstrating mixed Python/Rust module with user-managed re-exports.

Run with: pyright test_imports.py

This tests:
1. Re-exports from __init__.py (SomeClass, some_function from pure Python)
2. Re-exports from __init__.py (NativeClass, native_function from Rust)
3. Deep nested submodules via add_submodule
"""

from mixed_with_py_init import SomeClass
from mixed_with_py_init import some_function
from mixed_with_py_init import NativeClass
from mixed_with_py_init import native_function
from mixed_with_py_init._native.deep.nested.module import deep_function


def main() -> None:
    # Pure Python class and function
    obj = SomeClass("test")
    print(obj.greet())

    result = some_function(5)
    print(f"some_function(5) = {result}")

    # Rust class and function (re-exported via __init__.py)
    native = NativeClass(10)
    print(f"native.double() = {native.double()}")

    print(f"native_function(5) = {native_function(5)}")

    # Deep nested submodule
    print(f"deep_function() = {deep_function()}")


if __name__ == "__main__":
    main()
