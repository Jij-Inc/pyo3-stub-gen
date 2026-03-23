"""Test file to demonstrate the __init__.py vs __init__.pyi conflict.

Run with: pyright test_imports.py

Expected: All imports should work (as they do at runtime)
Actual: Type checker errors because __init__.pyi shadows __init__.py
"""

# These imports work at RUNTIME but FAIL type checking
# because __init__.pyi doesn't include the re-exports from __init__.py

from mixed_with_py_init import SomeClass  # Error: "SomeClass" is not exported
from mixed_with_py_init import some_function  # Error: "some_function" is not exported
from mixed_with_py_init import NativeClass  # Error: "NativeClass" is not exported
from mixed_with_py_init import native_function  # Error: "native_function" is not exported

# DirectClass is defined with module="mixed_with_py_init" in Rust,
# so it IS in __init__.pyi - this should work!
from mixed_with_py_init import DirectClass  # OK: DirectClass is in __init__.pyi


def main() -> None:
    # These would work at runtime
    obj = SomeClass("test")
    print(obj.greet())

    result = some_function(5)
    print(f"some_function(5) = {result}")

    native = NativeClass(10)
    print(f"native.double() = {native.double()}")

    print(f"native_function(5) = {native_function(5)}")

    # DirectClass works both at runtime and for type checking
    direct = DirectClass("world")
    print(direct.greet())


if __name__ == "__main__":
    main()
