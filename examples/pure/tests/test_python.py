from pure import (
    sum,
    create_dict,
    read_dict,
    echo_path,
    ahash_dict,
    NumberRich,
    Shape1,
    Shape2,
)
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
    i2 = NumberRich.INTEGER()
    assert i2.int == 2


# Test code for complex enum case from PyO3 document
# https://pyo3.rs/v0.25.1/class.html#complex-enums
def test_complex_enum_shape1():
    circle = Shape1.Circle(radius=10.0)
    square = Shape1.RegularPolygon(4, 10.0)

    assert isinstance(circle, Shape1)
    assert isinstance(circle, Shape1.Circle)
    assert circle.radius == 10.0

    assert isinstance(square, Shape1)
    assert isinstance(square, Shape1.RegularPolygon)
    assert square[0] == 4  # Gets _0 field
    assert square[1] == 10.0  # Gets _1 field

    def count_vertices(cls, shape):
        match shape:
            case cls.Circle():
                return 0
            case cls.Rectangle():
                return 4
            case cls.RegularPolygon(n):
                return n
            case cls.Nothing():
                return 0

    assert count_vertices(Shape1, circle) == 0
    assert count_vertices(Shape1, square) == 4


# Test code for complex enum case from PyO3 document
# https://pyo3.rs/v0.25.1/class.html#complex-enums
def test_complex_enum_shape2():
    circle = Shape2.Circle()
    assert isinstance(circle, Shape2)
    assert isinstance(circle, Shape2.Circle)
    assert circle.radius == 1.0

    square = Shape2.Rectangle(width=1, height=1)
    assert isinstance(square, Shape2)
    assert isinstance(square, Shape2.Rectangle)
    assert square.width == 1
    assert square.height == 1

    hexagon = Shape2.RegularPolygon(6)
    assert isinstance(hexagon, Shape2)
    assert isinstance(hexagon, Shape2.RegularPolygon)
    assert hexagon.side_count == 6
    assert hexagon.radius == 1


def test_path():
    out = echo_path(pathlib.Path("test"))
    assert out == pathlib.Path("test")

    out = echo_path("test")
    assert out == pathlib.Path("test")
