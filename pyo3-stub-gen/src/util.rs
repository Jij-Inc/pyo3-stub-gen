use pyo3::{prelude::*, types::*};

pub fn fmt_py_obj(any: &Bound<'_, PyAny>) -> String {
    fn all_builtin_types(any: &Bound<'_, PyAny>) -> bool {
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
    if all_builtin_types(any) {
        // `to_string` for `PyString` will lose quotes
        if any.is_instance_of::<PyString>() {
            if let Ok(s) = any.extract::<String>() {
                return format!("'{s}'");
            }
        } else {
            return any.to_string();
        }
    }
    "...".to_owned()
}

#[cfg(test)]
mod test {
    use pyo3::IntoPyObjectExt;

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
            assert_eq!("{'k1': 'v1', 'k2': 2}", fmt_py_obj(&dict));
            // class A variable can not be formatted
            _ = dict.set_item("k3", A {});
            assert_eq!("...", fmt_py_obj(&dict));
        })
    }
    #[test]
    fn test_fmt_list() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 2]).unwrap();
            assert_eq!("[1, 2]", fmt_py_obj(&list));
            // class A variable can not be formatted
            let list = PyList::new(py, [A {}, A {}]).unwrap();
            assert_eq!("...", fmt_py_obj(&list));
        })
    }
    #[test]
    fn test_fmt_tuple() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, [1, 2]).unwrap();
            assert_eq!("(1, 2)", fmt_py_obj(&tuple));
            let tuple = PyTuple::new(py, [1]).unwrap();
            assert_eq!("(1,)", fmt_py_obj(&tuple));
            // class A variable can not be formatted
            let tuple = PyTuple::new(py, [A {}]).unwrap();
            assert_eq!("...", fmt_py_obj(&tuple));
        })
    }
    #[test]
    fn test_fmt_other() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // str
            assert_eq!("'123'", fmt_py_obj(&"123".into_bound_py_any(py).unwrap()));
            // bool
            assert_eq!("True", fmt_py_obj(&true.into_bound_py_any(py).unwrap()));
            assert_eq!("False", fmt_py_obj(&false.into_bound_py_any(py).unwrap()));
            // int
            assert_eq!("123", fmt_py_obj(&123.into_bound_py_any(py).unwrap()));
            // float
            assert_eq!("1.23", fmt_py_obj(&1.23.into_bound_py_any(py).unwrap()));
            // None
            let none: Option<usize> = None;
            assert_eq!("None", fmt_py_obj(&none.into_bound_py_any(py).unwrap()));
            // class A variable can not be formatted
            assert_eq!("...", fmt_py_obj(&A {}.into_bound_py_any(py).unwrap()));
        })
    }
}
