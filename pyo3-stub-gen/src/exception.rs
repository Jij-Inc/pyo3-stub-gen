use pyo3::exceptions::*;

#[macro_export]
macro_rules! create_exception {
    ($module: expr, $name: ident, $base: ty) => {
        $crate::create_exception!($module, $name, $base, "");
    };
    ($module: expr, $name: ident, $base: ty, $doc: expr) => {
        ::pyo3::create_exception!($module, $name, $base, $doc);

        $crate::inventory::submit! {
            $crate::type_info::PyErrorInfo {
                error_id: ::std::any::TypeId::of::<$name>,
                name: stringify!($name),
                module: stringify!($module),
                base: <$base as $crate::exception::NativeException>::type_name,
            }
        }
    };
}

/// Native exceptions in Python
pub trait NativeException {
    /// Type name in Python side
    fn type_name() -> &'static str;
}

macro_rules! impl_native_exception {
    ($name:ident, $type_name:literal) => {
        impl NativeException for $name {
            fn type_name() -> &'static str {
                $type_name
            }
        }
    };
}

impl_native_exception!(PyArithmeticError, "ArithmeticError");
impl_native_exception!(PyAssertionError, "AssertionError");
impl_native_exception!(PyAttributeError, "AttributeError");
impl_native_exception!(PyBaseException, "BaseException");
impl_native_exception!(PyBlockingIOError, "BlockingIOError");
impl_native_exception!(PyBrokenPipeError, "BrokenPipeError");
impl_native_exception!(PyBufferError, "BufferError");
impl_native_exception!(PyBytesWarning, "BytesWarning");
impl_native_exception!(PyChildProcessError, "ChildProcessError");
impl_native_exception!(PyConnectionAbortedError, "ConnectionAbortedError");
impl_native_exception!(PyConnectionError, "ConnectionError");
impl_native_exception!(PyConnectionRefusedError, "ConnectionRefusedError");
impl_native_exception!(PyConnectionResetError, "ConnectionResetError");
impl_native_exception!(PyDeprecationWarning, "DeprecationWarning");
impl_native_exception!(PyEOFError, "EOFError");
// FIXME: This only exists in Python 3.10+.
//        We need to find a way to conditionally compile this.
// impl_native_exception!(PyEncodingWarning, "EncodingWarning");
impl_native_exception!(PyEnvironmentError, "EnvironmentError");
impl_native_exception!(PyException, "Exception");
impl_native_exception!(PyFileExistsError, "FileExistsError");
impl_native_exception!(PyFileNotFoundError, "FileNotFoundError");
impl_native_exception!(PyFloatingPointError, "FloatingPointError");
impl_native_exception!(PyFutureWarning, "FutureWarning");
impl_native_exception!(PyGeneratorExit, "GeneratorExit");
impl_native_exception!(PyIOError, "IOError");
impl_native_exception!(PyImportError, "ImportError");
impl_native_exception!(PyImportWarning, "ImportWarning");
impl_native_exception!(PyIndexError, "IndexError");
impl_native_exception!(PyInterruptedError, "InterruptedError");
impl_native_exception!(PyIsADirectoryError, "IsADirectoryError");
impl_native_exception!(PyKeyError, "KeyError");
impl_native_exception!(PyKeyboardInterrupt, "KeyboardInterrupt");
impl_native_exception!(PyLookupError, "LookupError");
impl_native_exception!(PyMemoryError, "MemoryError");
impl_native_exception!(PyModuleNotFoundError, "ModuleNotFoundError");
impl_native_exception!(PyNameError, "NameError");
impl_native_exception!(PyNotADirectoryError, "NotADirectoryError");
impl_native_exception!(PyNotImplementedError, "NotImplementedError");
impl_native_exception!(PyOSError, "OSError");
impl_native_exception!(PyOverflowError, "OverflowError");
impl_native_exception!(PyPendingDeprecationWarning, "PendingDeprecationWarning");
impl_native_exception!(PyPermissionError, "PermissionError");
impl_native_exception!(PyProcessLookupError, "ProcessLookupError");
impl_native_exception!(PyRecursionError, "RecursionError");
impl_native_exception!(PyReferenceError, "ReferenceError");
impl_native_exception!(PyResourceWarning, "ResourceWarning");
impl_native_exception!(PyRuntimeError, "RuntimeError");
impl_native_exception!(PyRuntimeWarning, "RuntimeWarning");
impl_native_exception!(PyStopAsyncIteration, "StopAsyncIteration");
impl_native_exception!(PyStopIteration, "StopIteration");
impl_native_exception!(PySyntaxError, "SyntaxError");
impl_native_exception!(PySyntaxWarning, "SyntaxWarning");
impl_native_exception!(PySystemError, "SystemError");
impl_native_exception!(PySystemExit, "SystemExit");
impl_native_exception!(PyTimeoutError, "TimeoutError");
impl_native_exception!(PyTypeError, "TypeError");
impl_native_exception!(PyUnboundLocalError, "UnboundLocalError");
impl_native_exception!(PyUnicodeDecodeError, "UnicodeDecodeError");
impl_native_exception!(PyUnicodeEncodeError, "UnicodeEncodeError");
impl_native_exception!(PyUnicodeError, "UnicodeError");
impl_native_exception!(PyUnicodeTranslateError, "UnicodeTranslateError");
impl_native_exception!(PyUnicodeWarning, "UnicodeWarning");
impl_native_exception!(PyUserWarning, "UserWarning");
impl_native_exception!(PyValueError, "ValueError");
impl_native_exception!(PyWarning, "Warning");
impl_native_exception!(PyZeroDivisionError, "ZeroDivisionError");
