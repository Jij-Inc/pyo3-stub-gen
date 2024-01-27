import pyo3_stub_gen_testing_mixed.my_rust_pkg


def test_sum_as_string():
    assert pyo3_stub_gen_testing_mixed.my_rust_pkg.sum_as_string(1, "2") == "3"
