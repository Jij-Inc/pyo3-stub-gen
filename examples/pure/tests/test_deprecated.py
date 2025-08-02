"""Test that deprecated annotations are correctly generated in stub files."""
import pure
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import typing_extensions

def test_deprecated_function_exists():
    """Test that deprecated function exists and is callable."""
    # The function should still be callable even though it's deprecated
    pure.deprecated_function()


def test_deprecated_method_exists():
    """Test that deprecated method exists and is callable."""
    a = pure.A(5)
    # The method should still be callable even though it's deprecated
    a.deprecated_method()


def test_stub_file_has_deprecated_decorators():
    """Verify that the stub file contains the deprecated decorators."""
    import pathlib
    stub_file = pathlib.Path(__file__).parent.parent / "pure.pyi"
    content = stub_file.read_text()
    
    # Check for deprecated function decorator
    assert '@typing_extensions.deprecated("since=1.0, This function is deprecated")' in content
    assert 'def deprecated_function() -> None:' in content
    
    # Check for deprecated method decorator
    assert '@typing_extensions.deprecated("since=1.0, This method is deprecated")' in content
    assert 'def deprecated_method(self) -> None:' in content
    
    # Check that typing_extensions is imported
    assert 'import typing_extensions' in content