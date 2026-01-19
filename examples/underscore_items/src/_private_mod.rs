// Private submodule (underscore-prefixed) for testing __all__ generation
// This submodule should be excluded from __all__ of parent module

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

// Public function in a private module
#[gen_stub_pyfunction(module = "underscore_items._private_mod")]
#[pyfunction]
pub fn hidden_function() {}

// Public class in a private module
#[gen_stub_pyclass]
#[pyclass(module = "underscore_items._private_mod")]
pub struct HiddenClass {}

/// Register functions and classes to the submodule
pub fn register_module(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hidden_function, m)?)?;
    m.add_class::<HiddenClass>()?;
    Ok(())
}
