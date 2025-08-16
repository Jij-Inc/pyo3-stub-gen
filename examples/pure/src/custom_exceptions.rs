use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
    types::*,
    PyObject,
};
use pyo3_stub_gen::{create_exception, derive::*};

// Use `create_exception!` to create a custom exception
create_exception!(pure, MyError, PyRuntimeError);

/// A manual custom exception case
///
/// Based on the code reported in https://github.com/Jij-Inc/pyo3-stub-gen/issues/263
#[gen_stub_pyclass]
#[pyclass(frozen, extends=PyTypeError)]
#[derive(Debug)]
pub struct NotIntError {
    item: PyObject,
}

impl<'py> IntoPyObject<'py> for NotIntError {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        py.get_type::<NotIntError>().call1((self.item,))
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl NotIntError {
    #[new]
    fn new(item: Bound<'_, PyAny>) -> NotIntError {
        NotIntError {
            item: item.unbind(),
        }
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        let item_type = self.item.bind(py).get_type().name().unwrap();
        Ok(format!(
            "Expected int but found `{}`, {}",
            item_type, self.item
        ))
    }

    /// A trivial number
    fn trivial_number(&self) -> PyResult<i16> {
        Ok(123)
    }

    /// Checks if the item is a string
    fn item_is_str(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.item.bind(py).is_instance_of::<PyString>())
    }
}
