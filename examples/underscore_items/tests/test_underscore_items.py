import underscore_items
import underscore_items._private_mod
from pathlib import Path


def read_stub_all(stub_path: Path) -> list[str]:
    """Read __all__ list from a stub file."""
    content = stub_path.read_text()
    # Find __all__ = [ ... ] pattern
    all_items = []
    in_all = False
    for line in content.split("\n"):
        if "__all__ = [" in line:
            in_all = True
            continue
        if in_all:
            if "]" in line:
                break
            # Extract quoted item
            line = line.strip()
            if line.startswith('"') and line.endswith('",'):
                all_items.append(line[1:-2])
    return all_items


def test_underscore_submodule_excluded():
    """Underscore-prefixed submodules should be excluded from __all__"""
    stub_path = Path(__file__).parent.parent / "python" / "underscore_items" / "__init__.pyi"
    all_items = read_stub_all(stub_path)
    assert "_private_mod" not in all_items


def test_explicit_inclusion():
    """Explicitly added underscore items should be in __all__"""
    stub_path = Path(__file__).parent.parent / "python" / "underscore_items" / "__init__.pyi"
    all_items = read_stub_all(stub_path)
    assert "_private_function" in all_items
    assert "_PrivateClass" in all_items


def test_explicit_exclusion():
    """Explicitly excluded public items should not be in __all__"""
    stub_path = Path(__file__).parent.parent / "python" / "underscore_items" / "__init__.pyi"
    all_items = read_stub_all(stub_path)
    assert "public_but_hidden" not in all_items
    # Verify the function is still defined in the stub file, just not in __all__
    stub_content = stub_path.read_text()
    assert "def public_but_hidden()" in stub_content


def test_public_items_included():
    """Public items (without underscore) should be in __all__ by default"""
    stub_path = Path(__file__).parent.parent / "python" / "underscore_items" / "__init__.pyi"
    all_items = read_stub_all(stub_path)
    assert "public_function" in all_items
    assert "PublicClass" in all_items


def test_private_submodule_contents():
    """Items in private submodule should have their own __all__"""
    stub_path = Path(__file__).parent.parent / "python" / "underscore_items" / "_private_mod" / "__init__.pyi"
    all_items = read_stub_all(stub_path)
    assert "hidden_function" in all_items
    assert "HiddenClass" in all_items
