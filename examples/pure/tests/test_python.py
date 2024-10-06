from pure import sum, create_dict, read_dict, echo_path, ahash_dict, add_two_number, calculate_average
import pytest
import pathlib


def test_sum():
    assert sum([1, 2]) == 3
    assert sum((1, 2)) == 3

def test_add_two_number():
    assert add_two_number() == 0
    assert add_two_number(10) == 10
    assert add_two_number(24, 36) == 60

def test_calculate_average():
    assert calculate_average([1,2,3,4]) == 2.5
    assert calculate_average([1,2,3,4], 2) == 2.50

def test_create_dict():
    assert create_dict(3) == {0: [], 1: [0], 2: [0, 1]}


def test_ahash_dict():
    assert ahash_dict() == {"apple": 3, "banana": 2, "orange": 5}


def test_read_dict():
    read_dict(
        {
            0: {
                0: 1,
            },
            1: {
                0: 2,
                1: 3,
            },
        }
    )

    with pytest.raises(TypeError) as e:
        read_dict({0: 1})  # type: ignore
    assert (
        str(e.value) == "argument 'dict': 'int' object cannot be converted to 'PyDict'"
    )


def test_path():
    out = echo_path(pathlib.Path("test"))
    assert out == "test"

    out = echo_path("test")
    assert out == "test"
