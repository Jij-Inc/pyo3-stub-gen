from pure import sum, create_dict, read_dict, echo_path, ahash_dict, NumberRich
import pytest
import pathlib


def test_sum():
    assert sum([1, 2]) == 3
    assert sum((1, 2)) == 3


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

def test_number_rich():
    i = NumberRich.INTEGER(1)
    f = NumberRich.FLOAT(1.5)
    assert i.int == 1
    assert f._0 == 1.5
    assert len(f) == 1
    match i:
        case NumberRich.INTEGER(n):
            assert n == 1
    match f:
        case NumberRich.FLOAT(n):
            assert n == 1.5
    i2 = NumberRich.INTEGER()
    assert i2.int == 2



def test_path():
    out = echo_path(pathlib.Path("test"))
    assert out == pathlib.Path("test")

    out = echo_path("test")
    assert out == pathlib.Path("test")
