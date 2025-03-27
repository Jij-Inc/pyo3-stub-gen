# This file is automatically generated by pyo3_stub_gen
# ruff: noqa: CPY001, D200, D212, E501, F401

import builtins
import os
import pathlib
import typing

MY_CONSTANT: builtins.int
class A:
    x: builtins.int
    def __new__(cls,x:builtins.int): ...
    def show_x(self) -> None:
        ...

    def ref_test(self, x:dict) -> dict:
        ...


class Number:
    FLOAT: Number
    r"""
    A floating point number.
    """
    INTEGER: Number
    r"""
    An integer.
    """

    def is_a_float(self) -> builtins.bool:
        ...

def ahash_dict() -> builtins.dict[builtins.str, builtins.int]:
    ...

def create_a(x:builtins.int=2) -> A:
    ...

def create_dict(n:builtins.int) -> builtins.dict[builtins.int, builtins.list[builtins.int]]:
    ...

def default_value(num:Number=...) -> Number:
    ...

def echo_path(path:builtins.str | os.PathLike | pathlib.Path) -> builtins.str:
    ...

def read_dict(dict:typing.Mapping[builtins.int, typing.Mapping[builtins.int, builtins.int]]) -> None:
    ...

def str_len(x:builtins.str) -> builtins.int:
    r"""
    Returns the length of the string.
    """
    ...

def sum(v:typing.Sequence[builtins.int]) -> builtins.int:
    r"""
    Returns the sum of two numbers as a string.
    """
    ...

class MyError(RuntimeError): ...

