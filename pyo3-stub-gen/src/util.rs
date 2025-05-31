use pyo3::{prelude::*, types::*};
use std::ffi::CString;

pub fn all_builtin_types(any: &Bound<'_, PyAny>) -> bool {
    if any.is_instance_of::<PyString>()
        || any.is_instance_of::<PyBool>()
        || any.is_instance_of::<PyInt>()
        || any.is_instance_of::<PyFloat>()
        || any.is_none()
    {
        return true;
    }
    if any.is_instance_of::<PyDict>() {
        return any
            .downcast::<PyDict>()
            .map(|dict| {
                dict.into_iter()
                    .all(|(k, v)| all_builtin_types(&k) && all_builtin_types(&v))
            })
            .unwrap_or(false);
    }
    if any.is_instance_of::<PyList>() {
        return any
            .downcast::<PyList>()
            .map(|list| list.into_iter().all(|v| all_builtin_types(&v)))
            .unwrap_or(false);
    }
    if any.is_instance_of::<PyTuple>() {
        return any
            .downcast::<PyTuple>()
            .map(|list| list.into_iter().all(|v| all_builtin_types(&v)))
            .unwrap_or(false);
    }
    false
}

/// whether eval(repr(any)) == any
pub fn valid_external_repr(any: &Bound<'_, PyAny>) -> Option<bool> {
    let globals = get_globals(any).ok()?;
    let fmt_str = any.repr().ok()?.to_string();
    let fmt_cstr = CString::new(fmt_str.clone()).ok()?;
    let new_any = any.py().eval(&fmt_cstr, Some(&globals), None).ok()?;
    new_any.eq(any).ok()
}

fn get_globals<'py>(any: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    let type_object = any.get_type();
    let type_name = type_object.getattr("__name__")?;
    let type_name: &str = type_name.extract()?;
    let globals = PyDict::new(any.py());
    globals.set_item(type_name, type_object)?;
    Ok(globals)
}

pub fn fmt_py_obj<'py, T: pyo3::IntoPyObjectExt<'py>>(py: Python<'py>, obj: T) -> String {
    if let Ok(any) = obj.into_bound_py_any(py) {
        if all_builtin_types(&any) || valid_external_repr(&any).is_some_and(|valid| valid) {
            if let Ok(py_str) = any.repr() {
                return py_str.to_string();
            }
        }
    }
    "...".to_owned()
}

#[cfg(test)]
mod test {
    use super::*;
    #[pyclass]
    #[derive(Debug)]
    struct A {}
    #[test]
    fn test_fmt_dict() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            _ = dict.set_item("k1", "v1");
            _ = dict.set_item("k2", 2);
            assert_eq!("{'k1': 'v1', 'k2': 2}", fmt_py_obj(py, &dict));
            // class A variable can not be formatted
            _ = dict.set_item("k3", A {});
            assert_eq!("...", fmt_py_obj(py, &dict));
        })
    }
    #[test]
    fn test_fmt_list() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 2]).unwrap();
            assert_eq!("[1, 2]", fmt_py_obj(py, &list));
            // class A variable can not be formatted
            let list = PyList::new(py, [A {}, A {}]).unwrap();
            assert_eq!("...", fmt_py_obj(py, &list));
        })
    }
    #[test]
    fn test_fmt_tuple() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, [1, 2]).unwrap();
            assert_eq!("(1, 2)", fmt_py_obj(py, tuple));
            let tuple = PyTuple::new(py, [1]).unwrap();
            assert_eq!("(1,)", fmt_py_obj(py, tuple));
            // class A variable can not be formatted
            let tuple = PyTuple::new(py, [A {}]).unwrap();
            assert_eq!("...", fmt_py_obj(py, tuple));
        })
    }
    #[test]
    fn test_fmt_other() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // str
            assert_eq!("'123'", fmt_py_obj(py, "123"));
            assert_eq!("\"don't\"", fmt_py_obj(py, "don't"));
            assert_eq!("'str\\\\'", fmt_py_obj(py, "str\\"));
            // bool
            assert_eq!("True", fmt_py_obj(py, true));
            assert_eq!("False", fmt_py_obj(py, false));
            // int
            assert_eq!("123", fmt_py_obj(py, 123));
            // float
            assert_eq!("1.23", fmt_py_obj(py, 1.23));
            // None
            let none: Option<usize> = None;
            assert_eq!("None", fmt_py_obj(py, none));
            // class A variable can not be formatted
            assert_eq!("...", fmt_py_obj(py, A {}));
        })
    }
    #[test]
    fn test_fmt_enum() {
        #[pyclass(eq, eq_int)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum Number {
            Float,
            Integer,
        }
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            assert_eq!("Number.Float", fmt_py_obj(py, Number::Float));
        });
    }
}
