#!/usr/bin/env python3
"""Test NumPy integration with pyo3-stub-gen."""

import numpy as np
import pure


def test_sum_array_1d():
    """Test summing 1D array - NDArray input to scalar output."""
    arr = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
    result = pure.sum_array_1d(arr)
    assert result == 15.0


def test_create_zeros_1d():
    """Test creating 1D zeros array - scalar input to NDArray output."""
    arr = pure.create_zeros_1d(5)
    assert arr.shape == (5,)
    assert np.all(arr == 0.0)


def test_int_to_float():
    """Test converting int array to float - NDArray[i32] to NDArray[f64]."""
    arr = np.array([1, 2, 3, 4], dtype=np.int32)
    result = pure.int_to_float(arr)
    expected = np.array([1.0, 2.0, 3.0, 4.0])
    np.testing.assert_array_equal(result, expected)


def test_process_float32_array():
    """Test processing float32 array - NDArray[f32] to NDArray[f32]."""
    arr = np.array([1.0, 2.0, 3.0], dtype=np.float32)
    result = pure.process_float32_array(arr)
    expected = np.array([2.0, 4.0, 6.0], dtype=np.float32)
    np.testing.assert_array_almost_equal(result, expected)


def test_sum_dynamic_array():
    """Test summing dynamic dimensional array - PyReadonlyArrayDyn."""
    # Test with 3D array
    arr = np.array([[[1.0, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]])
    result = pure.sum_dynamic_array(arr)
    assert result == 36.0

    # Test with 1D array
    arr = np.array([1.0, 2.0, 3.0])
    result = pure.sum_dynamic_array(arr)
    assert result == 6.0


def test_optional_array_param():
    """Test optional array parameter - Option<PyReadonlyArray1>."""
    # With array
    arr = np.array([1.0, 2.0, 3.0])
    result = pure.optional_array_param(arr)
    assert result == "Array with 3 elements"

    # Without array
    result = pure.optional_array_param(None)
    assert result == "No array provided"


def test_split_array():
    """Test splitting array - returning tuple of NDArrays."""
    arr = np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
    first, second = pure.split_array(arr)
    np.testing.assert_array_equal(first, np.array([1.0, 2.0, 3.0]))
    np.testing.assert_array_equal(second, np.array([4.0, 5.0, 6.0]))


def test_count_true():
    """Test counting true values - NDArray[bool] to int."""
    arr = np.array([True, False, True, True, False])
    result = pure.count_true(arr)
    assert result == 3


if __name__ == "__main__":
    # Run all tests
    test_sum_array_1d()
    test_create_zeros_1d()
    test_int_to_float()
    test_process_float32_array()
    test_sum_dynamic_array()
    test_optional_array_param()
    test_split_array()
    test_count_true()

    print("All NumPy tests passed!")
