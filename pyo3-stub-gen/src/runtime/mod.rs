//! Runtime support for type aliases.
//!
//! This module provides traits and utilities for creating Python type objects at runtime,
//! enabling type aliases defined with [`define_type_alias!`](crate::define_type_alias) to be
//! registered in Python modules.
//!
//! # Example
//!
//! ```rust,ignore
//! use pyo3::prelude::*;
//! use pyo3_stub_gen::define_type_alias;
//! use pyo3_stub_gen::runtime::PyModuleTypeAliasExt;
//!
//! // Define a runtime type alias
//! define_type_alias! {
//!     pub struct NumberOrString in "my_module"; i32 | String
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<PyModule>) -> PyResult<()> {
//!     // Register the type alias at runtime
//!     m.add_type_alias::<NumberOrString>()?;
//!     Ok(())
//! }
//! ```

mod builtins;
mod pyo3_types;

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
/// A Python type object representing the union of all input types.
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

/// Trait for obtaining Python type objects from Rust types at runtime.
///
/// This trait enables conversion from Rust types to their corresponding Python type objects,
/// which is necessary for creating union types at runtime.
///
/// # Implementation Notes
///
/// - For primitive types (i32, String, etc.), implementations return the corresponding
///   Python built-in types (int, str, etc.)
/// - For PyO3 classes (types implementing `PyTypeInfo`), implementations return the
///   Python class object
/// - Wrapper types like `Py<T>` and `Bound<T>` delegate to their inner type
pub trait PyRuntimeType {
    /// Returns the Python type object corresponding to this Rust type.
    ///
    /// # Arguments
    ///
    /// * `py` - Python interpreter token
    ///
    /// # Returns
    ///
    /// The Python type object (e.g., `<class 'int'>` for `i32`)
    fn py_type(py: Python<'_>) -> PyResult<Bound<'_, PyAny>>;
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
            let int_type = py.get_type::<PyInt>().into_any();
            let result = union_type(py, &[int_type.clone()]);
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
            if let Err(ref e) = result {
                eprintln!("Error: {:?}", e);
            }
            assert!(result.is_ok(), "union_type failed: {:?}", result);
            // The result should be a union type (int | str)
            let union = result.unwrap();
            // Check that it's a UnionType by checking its repr
            let repr = union.repr().unwrap().to_string();
            assert!(repr.contains("int") && repr.contains("str"));
        });
    }
}
