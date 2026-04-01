//! Runtime support for type aliases.
//!
//! This module provides traits and utilities for registering type aliases
//! in Python modules at runtime, enabling type aliases defined with
//! [`define_type_alias!`](crate::define_type_alias) to be importable from Python.
//!
//! # Example
//!
//! ```rust,ignore
//! use pyo3::prelude::*;
//! use pyo3::types::{PyInt, PyString};
//! use pyo3_stub_gen::define_type_alias;
//! use pyo3_stub_gen::runtime::PyModuleTypeAliasExt;
//!
//! // Define a runtime type alias using Python types
//! define_type_alias! {
//!     pub struct NumberOrString in "my_module"; PyInt | PyString
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<PyModule>) -> PyResult<()> {
//!     // Register the type alias at runtime
//!     m.add_type_alias::<NumberOrString>()?;
//!     Ok(())
//! }
//! ```

use ::pyo3::prelude::*;
use ::pyo3::types::PyModule;

/// Creates a Python union type using the `|` operator (Python 3.10+).
///
/// # Arguments
///
/// * `py` - Python interpreter token
/// * `types` - Slice of Python type objects to combine into a union
///
/// # Returns
///
/// A Python object representing the union of all input types.
/// For a single type, returns that type unchanged.
/// For multiple types, returns a `types.UnionType`.
///
/// # Errors
///
/// Returns an error if:
/// - The types slice is empty
/// - Any of the `__or__` operations fail
///
/// # Example
///
/// ```rust,ignore
/// use pyo3::types::{PyInt, PyString};
///
/// let union = union_type(py, &[
///     py.get_type::<PyInt>().into_any(),
///     py.get_type::<PyString>().into_any(),
/// ])?;
/// // union is equivalent to `int | str` in Python
/// ```
pub fn union_type<'py>(
    py: Python<'py>,
    types: &[Bound<'py, PyAny>],
) -> PyResult<Bound<'py, PyAny>> {
    if types.is_empty() {
        return Err(PyErr::new::<::pyo3::exceptions::PyValueError, _>(
            "union_type requires at least one type",
        ));
    }

    // Use Python's operator module to create union types
    // operator.or_(type1, type2) works correctly with type objects
    let operator = py.import("operator")?;
    let or_fn = operator.getattr("or_")?;

    let mut result = types[0].clone();
    for ty in &types[1..] {
        result = or_fn.call1((&result, ty))?;
    }
    Ok(result)
}

/// Trait for type aliases that can be registered at runtime.
///
/// This trait is automatically implemented by the [`define_type_alias!`](crate::define_type_alias)
/// macro. It provides the metadata and factory method needed to register a type alias
/// in a Python module.
///
/// # Associated Constants
///
/// * `NAME` - The Python name of the type alias
/// * `MODULE` - The module where the type alias is defined
///
/// # Required Methods
///
/// * `create_type_object` - Creates the Python type object representing the alias
pub trait PyTypeAlias: crate::PyStubType {
    /// The name of the type alias in Python.
    const NAME: &'static str;

    /// The module path where this type alias is defined.
    const MODULE: &'static str;

    /// Creates the Python type object for this type alias.
    ///
    /// For union types, this creates a union using the `|` operator.
    ///
    /// # Arguments
    ///
    /// * `py` - Python interpreter token
    ///
    /// # Returns
    ///
    /// The Python type object representing this type alias.
    fn create_type_object(py: Python<'_>) -> PyResult<Bound<'_, PyAny>>;
}

/// Extension trait for `Bound<PyModule>` to add type aliases.
///
/// This trait provides a convenient method for registering type aliases in Python modules.
///
/// # Example
///
/// ```rust,ignore
/// use pyo3::prelude::*;
/// use pyo3_stub_gen::runtime::PyModuleTypeAliasExt;
///
/// #[pymodule]
/// fn my_module(m: &Bound<PyModule>) -> PyResult<()> {
///     m.add_type_alias::<MyTypeAlias>()?;
///     Ok(())
/// }
/// ```
pub trait PyModuleTypeAliasExt {
    /// Adds a type alias to this module.
    ///
    /// The type alias will be available as a module attribute with the name
    /// specified by `T::NAME`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A type implementing [`PyTypeAlias`]
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Creating the type object fails
    /// - Adding the attribute to the module fails
    fn add_type_alias<T: PyTypeAlias>(&self) -> PyResult<()>;
}

impl PyModuleTypeAliasExt for Bound<'_, PyModule> {
    fn add_type_alias<T: PyTypeAlias>(&self) -> PyResult<()> {
        let type_object = T::create_type_object(self.py())?;
        self.add(T::NAME, type_object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::pyo3::type_object::PyTypeInfo;

    #[test]
    fn test_union_type_empty() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            let result = union_type(py, &[]);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_union_type_single() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            use ::pyo3::types::PyInt;
            let int_type = py.get_type::<PyInt>();
            let result = union_type(py, &[int_type.clone().into_any()]);
            assert!(result.is_ok());
            // Single type should just return itself
            let union = result.unwrap();
            assert!(union.is(&int_type));
        });
    }

    #[test]
    fn test_union_type_multiple() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            use ::pyo3::types::{PyInt, PyString};
            let int_type = py.get_type::<PyInt>().into_any();
            let str_type = py.get_type::<PyString>().into_any();
            let result = union_type(py, &[int_type, str_type]);
            assert!(result.is_ok(), "union_type failed: {:?}", result);
            // The result should be a union type (int | str)
            let union = result.unwrap();
            // Check that it's a UnionType by checking its repr
            let repr = union.repr().unwrap().to_string();
            assert!(repr.contains("int") && repr.contains("str"));
        });
    }

    // Test custom #[pyclass] with union_type
    #[::pyo3::pyclass]
    struct TestCustomClass {
        #[allow(dead_code)]
        value: i32,
    }

    #[test]
    fn test_union_type_with_pyclass() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            use ::pyo3::types::PyInt;
            let int_type = PyInt::type_object(py).into_any();
            let custom_type = TestCustomClass::type_object(py).into_any();
            let result = union_type(py, &[int_type, custom_type]);
            assert!(result.is_ok(), "union_type with pyclass failed: {:?}", result);
            // The result should be a union type (int | TestCustomClass)
            let union = result.unwrap();
            let repr = union.repr().unwrap().to_string();
            assert!(
                repr.contains("int") && repr.contains("TestCustomClass"),
                "Expected union repr to contain 'int' and 'TestCustomClass', got: {}",
                repr
            );
        });
    }

    // Test define_type_alias! macro with custom #[pyclass]
    #[::pyo3::pyclass]
    struct MyCustomType;

    // PyStubType implementation is required for define_type_alias!
    impl crate::PyStubType for MyCustomType {
        fn type_output() -> crate::TypeInfo {
            crate::TypeInfo::builtin("MyCustomType")
        }
        fn type_object(py: Python<'_>) -> ::pyo3::PyResult<Bound<'_, ::pyo3::PyAny>> {
            // Use PyTypeInfo for #[pyclass] types
            Ok(py.get_type::<Self>().into_any())
        }
    }

    crate::define_type_alias! {
        /// A union of a custom pyclass and int (using Rust type i32).
        pub struct CustomTypeOrInt in "test_module"; MyCustomType | i32
    }

    #[test]
    fn test_define_type_alias_with_pyclass() {
        pyo3::Python::initialize();
        Python::attach(|py| {
            let type_obj = CustomTypeOrInt::create_type_object(py);
            assert!(
                type_obj.is_ok(),
                "create_type_object failed: {:?}",
                type_obj
            );
            let union = type_obj.unwrap();
            let repr = union.repr().unwrap().to_string();
            assert!(
                repr.contains("MyCustomType") && repr.contains("int"),
                "Expected union repr to contain 'MyCustomType' and 'int', got: {}",
                repr
            );
        });
    }

    #[test]
    fn test_py_type_alias_constants() {
        assert_eq!(CustomTypeOrInt::NAME, "CustomTypeOrInt");
        assert_eq!(CustomTypeOrInt::MODULE, "test_module");
    }
}
