import empty_super_module


def test_empty_super_module():
    """Test that we can access submodules of empty parent modules."""
    empty_super_module.sub_mod.greet()


def test_deeply_nested_empty_modules():
    """Test that we can access deeply nested submodules through empty parent modules."""
    empty_super_module.deep.nested.module.deep_function()