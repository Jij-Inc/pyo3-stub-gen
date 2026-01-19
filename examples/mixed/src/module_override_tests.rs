//! Test cases for module attribute priority:
//! Priority order: inline > pyo3 > default
//!
//! Note: Testing only inline module parameters for now, as standalone
//! #[gen_stub(module = "...")] would require a new proc macro attribute

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

// ============================================================================
// PyClass Tests
// ============================================================================

/// Test 1: No module attribute → uses default
#[gen_stub_pyclass]
#[pyclass]
#[derive(Debug, Clone)]
pub struct NoModuleAttr {
    pub x: usize,
}

/// Test 2: Only pyo3 module → uses pyo3 module
#[gen_stub_pyclass]
#[pyclass(module = "mixed.test.from_pyo3")]
#[derive(Debug, Clone)]
pub struct OnlyPyo3Module {
    pub x: usize,
}

/// Test 3: Inline parameter overrides pyo3
#[gen_stub_pyclass(module = "mixed.test.from_inline")]
#[pyclass(module = "mixed.test.from_pyo3")]
#[derive(Debug, Clone)]
pub struct InlineOverridesPyo3Class {
    pub x: usize,
}

/// Test 4: Only inline (no pyo3)
#[gen_stub_pyclass(module = "mixed.test.from_inline")]
#[pyclass]
#[derive(Debug, Clone)]
pub struct OnlyInlineClass {
    pub x: usize,
}

// ============================================================================
// PyFunction Tests
// ============================================================================

/// Test 1: No module → uses default
#[gen_stub_pyfunction]
#[pyfunction]
pub fn no_module_fn() -> usize {
    42
}

/// Test 2: Only inline module
#[gen_stub_pyfunction(module = "mixed.test.from_inline")]
#[pyfunction]
pub fn only_inline_fn() -> usize {
    42
}

// ============================================================================
// PyEnum Tests
// ============================================================================

/// Test 1: Only pyo3 module
#[gen_stub_pyclass_enum]
#[pyclass(module = "mixed.test.from_pyo3")]
#[derive(Debug, Clone)]
pub enum OnlyPyo3Enum {
    A,
    B,
}

/// Test 2: Inline overrides pyo3
#[gen_stub_pyclass_enum(module = "mixed.test.from_inline")]
#[pyclass(module = "mixed.test.from_pyo3")]
#[derive(Debug, Clone)]
pub enum InlineOverridesPyo3Enum {
    A,
    B,
}

/// Test 3: Only inline (no pyo3)
#[gen_stub_pyclass_enum(module = "mixed.test.from_inline")]
#[pyclass]
#[derive(Debug, Clone)]
pub enum OnlyInlineEnum {
    A,
    B,
}
