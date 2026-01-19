// Test example for underscore-prefixed items in __all__ generation

mod _private_mod;

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

// 1. Test: Underscore-prefixed function should be excluded by default
#[gen_stub_pyfunction]
#[pyfunction]
fn _private_function() {}

// 2. Test: Underscore-prefixed class should be excluded by default
#[gen_stub_pyclass]
#[pyclass(module = "underscore_items")]
struct _PrivateClass {}

// 3. Test: Public function that should be in __all__ by default
#[gen_stub_pyfunction]
#[pyfunction]
fn public_function() {}

// 4. Test: Public class that should be in __all__ by default
#[gen_stub_pyclass]
#[pyclass]
struct PublicClass {}

// 5. Test: Public function that will be explicitly excluded from __all__
#[gen_stub_pyfunction]
#[pyfunction]
fn public_but_hidden() {}

// 6. Test: Explicit inclusion of underscore items via export_verbatim!
pyo3_stub_gen::export_verbatim!("underscore_items", "_private_function");
pyo3_stub_gen::export_verbatim!("underscore_items", "_PrivateClass");

// 7. Test: Explicit exclusion of public items via exclude_from_all!
pyo3_stub_gen::exclude_from_all!("underscore_items", "public_but_hidden");

/// Initializes the Python module
#[pymodule]
fn underscore_items(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_private_function, m)?)?;
    m.add_function(wrap_pyfunction!(public_function, m)?)?;
    m.add_function(wrap_pyfunction!(public_but_hidden, m)?)?;
    m.add_class::<_PrivateClass>()?;
    m.add_class::<PublicClass>()?;

    // Add the private submodule
    let py = m.py();
    let submod = PyModule::new(py, "_private_mod")?;
    _private_mod::register_module(&submod)?;
    m.add_submodule(&submod)?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
