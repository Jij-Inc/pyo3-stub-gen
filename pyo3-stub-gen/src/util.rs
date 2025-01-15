use pyo3::{prelude::*, types::*};

/// Reference to https://github.com/Jij-Inc/serde-pyobject/blob/5916fefc5b7f0e096ed13869dc41b36ad6f62dc2/src/de.rs#L309-L333
pub fn fmt_py_obj(any: &Bound<'_, PyAny>) -> Result<String, ()> {
    if any.is_instance_of::<PyDict>() {
        return any.downcast::<PyDict>().map_err(|_| ()).and_then(|dict| {
            dict.into_iter()
                .map(|(k, v)| {
                    if let (Ok(k), Ok(v)) = (fmt_py_obj(&k), fmt_py_obj(&v)) {
                        Ok(format!("{k}: {v}"))
                    } else {
                        Err(())
                    }
                })
                .collect::<Result<Vec<String>, ()>>()
                .map(|kv_list| format!("{{{}}}", kv_list.join(", ")))
        });
    }
    if any.is_instance_of::<PyList>() {
        return any.downcast::<PyList>().map_err(|_| ()).and_then(|list| {
            list.into_iter()
                .map(|v| fmt_py_obj(&v))
                .collect::<Result<Vec<String>, ()>>()
                .map(|v_list| format!("[{}]", v_list.join(", ")))
        });
    }
    if any.is_instance_of::<PyTuple>() {
        return any.downcast::<PyTuple>().map_err(|_| ()).and_then(|list| {
            list.into_iter()
                .map(|v| fmt_py_obj(&v))
                .collect::<Result<Vec<String>, ()>>()
                .map(|v_list| format!("({})", v_list.join(", ")))
        });
    }
    if any.is_instance_of::<PyString>() {
        return any
            .extract::<String>()
            .map_err(|_| ())
            .map(|s| format!("\"{s}\""));
    }
    if any.is_instance_of::<PyBool>() {
        // must be match before PyLong
        return any
            .extract::<bool>()
            .map_err(|_| ())
            .map(|b| if b { "True" } else { "False" }.to_owned());
    }
    if any.is_instance_of::<PyInt>() {
        return any.extract::<i64>().map_err(|_| ()).map(|n| n.to_string());
    }
    if any.is_instance_of::<PyFloat>() {
        return any.extract::<f64>().map_err(|_| ()).map(|f| f.to_string());
    }
    if any.is_none() {
        return Ok("None".to_owned());
    }
    Err(())
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
            assert_eq!(
                Ok("{\"k1\": \"v1\", \"k2\": 2}".to_owned()),
                fmt_py_obj(&dict)
            );
            // class A variable can not be formatted
            _ = dict.set_item("k3", A {});
            assert_eq!(Err(()), fmt_py_obj(&dict));
        })
    }
    #[test]
    fn test_fmt_list() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 2]).unwrap();
            assert_eq!(Ok("[1, 2]".to_owned()), fmt_py_obj(&list));
            // class A variable can not be formatted
            let list = PyList::new(py, [A {}, A {}]).unwrap();
            assert_eq!(Err(()), fmt_py_obj(&list));
        })
    }
    #[test]
    fn test_fmt_tuple() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, [1, 2]).unwrap();
            assert_eq!(Ok("(1, 2)".to_owned()), fmt_py_obj(&tuple));
            // class A variable can not be formatted
            let tuple = PyTuple::new(py, [A {}]).unwrap();
            assert_eq!(Err(()), fmt_py_obj(&tuple));
        })
    }
    #[test]
    fn test_fmt_other() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // str
            assert_eq!(Ok("\"123\"".to_owned()), fmt_py_obj(&"123".into_bound_py_any(py).unwrap()));
            // bool
            assert_eq!(Ok("True".to_owned()), fmt_py_obj(&true.into_bound_py_any(py).unwrap()));
            assert_eq!(Ok("False".to_owned()), fmt_py_obj(&false.into_bound_py_any(py).unwrap()));
            // int
            assert_eq!(Ok("123".to_owned()), fmt_py_obj(&123.into_bound_py_any(py).unwrap()));
            // float
            assert_eq!(Ok("1.23".to_owned()), fmt_py_obj(&1.23.into_bound_py_any(py).unwrap()));
            // None
            let none: Option<usize>  = None;
            assert_eq!(Ok("None".to_owned()), fmt_py_obj(&none.into_bound_py_any(py).unwrap()));
            // class A variable can not be formatted
            assert_eq!(Err(()), fmt_py_obj(&A{}.into_bound_py_any(py).unwrap()));
        })
    }
}
