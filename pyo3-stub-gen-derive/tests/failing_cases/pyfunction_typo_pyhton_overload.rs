use pyo3::prelude::*;
use pyo3_stub_gen_derive::gen_stub_pyfunction;

#[gen_stub_pyfunction(pyhton_overload = r#"
def example(x: int) -> int: ...
"#)]
#[pyfunction]
fn example(x: i32) -> i32 {
    x
}
