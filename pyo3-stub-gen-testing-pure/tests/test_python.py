from pyo3_stub_gen_testing_pure import sum


def test_sum():
    assert sum([1, 2]) == 3
    assert sum((1, 2)) == 3
