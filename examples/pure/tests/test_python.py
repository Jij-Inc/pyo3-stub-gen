from pure import (
    sum,
    create_a,
    create_dict,
    read_dict,
    echo_path,
    ahash_dict,
    async_num,
    NumberComplex,
    Shape1,
    Shape2,
    ComparableStruct,
    HashableStruct,
    add_decimals,
    DecimalHolder,
    fn_override_type,
    fn_with_python_param,
    fn_with_python_stub,
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


def test_number_complex():
    i = NumberComplex.INTEGER(1)
    f = NumberComplex.FLOAT(1.5)
    assert i.int == 1
    assert f._0 == 1.5
    assert len(f) == 1
    i2 = NumberComplex.INTEGER()
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


def test_overload_example_1():
    from pure import overload_example_1

    assert overload_example_1(1) == 2
    assert overload_example_1(1.5) == 2.5


def test_overload_example_2():
    from pure import overload_example_2

    assert overload_example_2(1) == 2
    assert overload_example_2(1.5) == 2.5


def test_overload_incrementer():
    from pure import Incrementer

    incr = Incrementer()

    assert incr.increment_1(1.5) == 2.5
    assert incr.increment_1(1) == 2


def test_overload_incrementer_2():
    from pure import Incrementer2

    incr = Incrementer2()

    assert incr.increment_2(1.5) == 3.5
    assert incr.increment_2(1) == 3


@pytest.mark.asyncio
async def test_async():
    assert await async_num() == 123
    a = create_a(1337)
    assert await a.async_get_x() == 1337


def test_comparable_struct_comparison_methods():
    """Test that comparison methods work correctly for pyclass(eq, ord).
    This verifies that issue #233 has been fixed."""

    # Create test instances
    a = ComparableStruct(5)
    b = ComparableStruct(10)
    c = ComparableStruct(5)

    # Test equality (__eq__)
    assert a == c
    assert not (a == b)
    assert a != b
    assert not (a != c)

    # Test ordering (__lt__, __le__, __gt__, __ge__)
    assert a < b
    assert not (b < a)
    assert not (a < c)

    assert a <= b
    assert a <= c
    assert not (b <= a)

    assert b > a
    assert not (a > b)
    assert not (a > c)

    assert b >= a
    assert a >= c
    assert not (a >= b)


def test_hashable_struct_hash_str_methods():
    """Test that the HashableStruct has hash and str methods"""
    obj1 = HashableStruct("test")
    obj2 = HashableStruct("test")
    obj3 = HashableStruct("other")

    # Test equality (required for hash)
    assert obj1 == obj2
    assert obj1 != obj3

    # Test hash
    assert hash(obj1) == hash(obj2)
    # Different objects might have the same hash, but equal objects must have the same hash

    # Test str
    assert str(obj1) == "HashableStruct(test)"
    assert str(obj3) == "HashableStruct(other)"

    # Test that it can be used in a set (requires hash)
    s = {obj1, obj2, obj3}
    assert len(s) == 2  # obj1 and obj2 are equal, so only 2 unique items


def test_add_decimals():
    """Test the add_decimals function works correctly"""
    from decimal import Decimal

    # Test basic addition
    result = add_decimals(Decimal("10.50"), Decimal("5.25"))
    assert result == Decimal("15.75")


def test_decimal_holder():
    """Test the DecimalHolder class can be created and its value returns what we expect"""
    from decimal import Decimal

    # Test creating a DecimalHolder
    holder = DecimalHolder(Decimal("123.45"))
    assert holder.value == Decimal("123.45")


def test_fn_override_type():
    """Test fn_override_type with callable argument"""
    def callback(s: str) -> int:
        return len(s)

    result = fn_override_type(callback)
    assert result == callback


def test_fn_with_python_param():
    """Test fn_with_python_param using python parameter in gen_stub_pyfunction"""
    def callback(s: str) -> int:
        return len(s)

    result = fn_with_python_param(callback)
    assert result == callback


def test_fn_with_python_stub():
    """Test fn_with_python_stub using gen_function_from_python! macro"""
    def callback(s: str) -> int:
        return len(s)

    result = fn_with_python_stub(callback)
    assert result == callback
