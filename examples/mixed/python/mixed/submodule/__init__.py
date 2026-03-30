"""Pure Python submodule for mixed example.

This module demonstrates that pure Python code can coexist with
PyO3-generated Rust modules in a mixed layout project.
"""


class SomeClass:
    """A pure Python class."""

    def __init__(self, name: str) -> None:
        self.name = name

    def greet(self) -> str:
        return f"Hello, {self.name}!"


def some_function(x: int) -> int:
    """A pure Python function."""
    return x * 2
