use pyo3::prelude::*;
use pyo3_stub_gen_derive::gen_stub_pyfunction;

#[gen_stub_pyfunction(no_default_overload = true)]
#[pyfunction]
fn example(x: i32) -> i32 {
    x
}
