use pyo3::prelude::*;
use pyo3_stub_gen::{derive::gen_stub_pyfunction, StubInfo};
use std::{env, path::*};

/// Gather information to generate stub files
pub fn stub_info() -> pyo3_stub_gen::Result<StubInfo> {
    let manifest_dir: &Path = env!("CARGO_MANIFEST_DIR").as_ref();
    StubInfo::from_pyproject_toml(manifest_dir.join("pyproject.toml"))
}

/// Returns the sum of two numbers as a string.
#[gen_stub_pyfunction]
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// Initializes the Python module
#[pymodule]
fn my_rust_pkg(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}

/// Test of unit test for testing link problem
#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
