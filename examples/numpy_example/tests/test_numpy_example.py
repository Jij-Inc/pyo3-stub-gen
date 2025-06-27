
from numpy_example import np_allow_type_change, np_type_must_match
import numpy as np
from numpy.testing import assert_array_equal
from pytest import raises

import json
import subprocess

from inspect import currentframe, getframeinfo



def assert_np_eq(a, b):
    assert_array_equal(a, b)
    assert a.dtype == b.dtype

def test_np_allow_type_change_can_accept_any_type_asarray_accepts():
    expected_result = np.array([1.0, 2.0, 3.0], dtype=np.float64)

    python_int_list: list[int] = [1, 2, 3]
    np_int_array = np.array([1, 2, 3], dtype=np.int32)
    np_f32_array = np.array([1.0, 2.0, 3.0], dtype=np.float32)
    tuple_int_list: tuple[int, int, int] = (1, 2, 3)
    tuple_float_list: tuple[float, float, float] = (1.0, 2.0, 3.0)

    assert_np_eq(np_allow_type_change(python_int_list), expected_result)
    assert_np_eq(np_allow_type_change(np_int_array), expected_result)
    assert_np_eq(np_allow_type_change(np_f32_array), expected_result)
    assert_np_eq(np_allow_type_change(tuple_int_list), expected_result)
    assert_np_eq(np_allow_type_change(tuple_float_list), expected_result)


_ERROR_LINES_BEGIN = -1
_ERROR_LINES_END = -1

def current_lineno(frame):
    assert frame is not None
    return getframeinfo(frame).lineno


def test_type_must_match_does_not_allow_type_change():
    expected_result = np.array([1, 2, 3], dtype=np.int16)

    python_int_list: list[int] = [1, 2, 131_072]
    np_int_array = np.array([1, 2, 131_072], dtype=np.int32)
    np_f32_array = np.array([1.0, 2.0, 131_072.0], dtype=np.float32)
    tuple_int_list: tuple[int, int, int] = (1, 2, 131_072)
    tuple_float_list: tuple[float, float, float] = (1.0, 2.0, 131_072.0)

    # These lines should trigger pyright errors
    global _ERROR_LINES_BEGIN
    global _ERROR_LINES_END

    _ERROR_LINES_BEGIN = current_lineno(currentframe())
    with raises(TypeError):
        assert_np_eq(np_type_must_match(python_int_list), expected_result)
    with raises(TypeError):
        assert_np_eq(np_type_must_match(np_int_array), expected_result)
    with raises(TypeError):
        assert_np_eq(np_type_must_match(np_f32_array), expected_result)
    with raises(TypeError):
        assert_np_eq(np_type_must_match(tuple_int_list), expected_result)
    with raises(TypeError):
        assert_np_eq(np_type_must_match(tuple_float_list), expected_result)
    _ERROR_LINES_END = current_lineno(currentframe())


def test_pyright_catches_errors_from_TypeMustMatch_functions():
    global _ERROR_LINES_BEGIN
    global _ERROR_LINES_END

    result = subprocess.run(["pyright", __file__, "--outputjson"], stdout=subprocess.PIPE)
    assert result.returncode == 1, "Pyright expected to fail, but it passed."
    report = json.loads(result.stdout.decode())

    assert report["summary"]["errorCount"] == 5, "Expected 5 errors"

    for diag in report["generalDiagnostics"]:
        assert diag["rule"] == "reportArgumentType"
        start_line = diag["range"]["start"]["line"]
        end_line = diag["range"]["end"]["line"]
        assert start_line > _ERROR_LINES_BEGIN, f"Expected start line {start_line} to be greater than {_ERROR_LINES_BEGIN} in {diag}"
        assert end_line < _ERROR_LINES_END, f"Expected end line {end_line} to be less than {_ERROR_LINES_END} in {diag}"


