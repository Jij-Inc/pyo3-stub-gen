use pyo3::{prelude::*, IntoPyObjectExt};
use pyo3_stub_gen::derive::*;

#[gen_stub_pyfunction]
#[pyfunction]
#[gen_stub(override_return_type(type_repr="collections.abc.Callable[[str], typing.Any]", imports=("collections.abc", "typing")))]
pub fn fn_override_type<'a>(
    #[gen_stub(override_type(type_repr="collections.abc.Callable[[str], typing.Any]", imports=("collections.abc", "typing")))]
    cb: Bound<'a, PyAny>,
) -> PyResult<Bound<'a, PyAny>> {
    cb.call1(("Hello!",))?;
    Ok(cb)
}

// Example: Using python parameter in gen_stub_pyfunction attribute
// This allows you to specify type information using Python stub syntax
#[gen_stub_pyfunction(python = r#"
    import collections.abc
    import typing

    def fn_with_python_param(callback: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
        """
        Example using python parameter in gen_stub_pyfunction attribute.
        This demonstrates specifying types directly in Python stub syntax.
        """
"#)]
#[pyfunction]
pub fn fn_with_python_param<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
    callback.call1(("Option C!",))?;
    Ok(callback)
}

// New example using gen_function_from_python!
#[pyfunction]
pub fn fn_with_python_stub<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
    callback.call1(("World!",))?;
    Ok(callback)
}

pyo3_stub_gen::inventory::submit! {
    pyo3_stub_gen::derive::gen_function_from_python! {
        r#"
        import collections.abc
        import typing

        def fn_with_python_stub(callback: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
            """
            Example function using gen_function_from_python! macro.
            This demonstrates how to define type information using Python stub syntax.
            """
        "#
    }
}
#[gen_stub_pyclass]
#[pyclass]
pub struct OverrideType {
    num: isize,
}

#[gen_stub_pymethods]
#[pymethods]
impl OverrideType {
    #[gen_stub(override_return_type(type_repr="typing_extensions.Never", imports=("typing_extensions")))]
    fn error(&self) -> PyResult<()> {
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "I'm an error!",
        ))
    }

    #[getter]
    #[gen_stub(override_return_type(type_repr = "int"))]
    fn get_num(&self) -> PyResult<Py<PyAny>> {
        Python::attach(|py| self.num.into_py_any(py))
    }

    #[setter]
    fn set_num(
        &mut self,
        #[gen_stub(override_type(type_repr = "str"))] value: Py<PyAny>,
    ) -> PyResult<()> {
        self.num = Python::attach(|py| value.extract::<String>(py))?.parse::<isize>()?;
        Ok(())
    }
}
