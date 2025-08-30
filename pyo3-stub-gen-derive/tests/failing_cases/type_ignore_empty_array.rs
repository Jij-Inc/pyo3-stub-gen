use pyo3::prelude::*;
use pyo3_stub_gen_derive::*;

#[gen_stub_pyfunction]
#[pyfunction]
#[gen_stub(type_ignore=[])]
fn test_function() -> PyResult<()> {
    Ok(())
}

fn main() {}