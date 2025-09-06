from mixed_sub_multiple import main_mod


def test_main_mod():
    """Test main module functions"""
    main_mod.greet_main()


def test_sub_mod_a():
    """Test mod_a functions"""
    main_mod.mod_a.greet_a()


def test_sub_mod_b():
    """Test mod_b functions"""
    main_mod.mod_b.greet_b()


def test_classes_in_main():
    """Test classes A and B in main module"""
    # Create instances
    a = main_mod.create_a(10)
    b = main_mod.create_b(20)
    
    # Test methods
    a.show_x()
    b.show_x()
    
    # Check types
    assert isinstance(a, main_mod.A)
    assert isinstance(b, main_mod.B)


def test_cross_module_references():
    """Test class C in mod_a that uses types A and B from main module"""
    # Create A and B instances
    a = main_mod.create_a(100)
    b = main_mod.create_b(200)
    
    # Create C instance that uses A and B
    c = main_mod.mod_a.create_c(a, b)
    c.show_x()
    
    # Check type
    assert isinstance(c, main_mod.mod_a.C)


def test_simple_class_in_mod_b():
    """Test class D in mod_b"""
    d = main_mod.mod_b.create_d(42)
    d.show_x()
    
    # Check type
    assert isinstance(d, main_mod.mod_b.D)


def test_int_submodule():
    """Test the int submodule (namespace collision test)"""
    result = main_mod.int.dummy_int_fun(123)
    assert result == 123


def test_type_annotations():
    """Test that type annotations are correctly generated in stub files"""
    # This test ensures the stub files are generated correctly
    # The actual type checking is done by pyright
    
    # These should work with proper type hints
    a: main_mod.A = main_mod.create_a(1)
    b: main_mod.B = main_mod.create_b(2)
    c: main_mod.mod_a.C = main_mod.mod_a.create_c(a, b)
    d: main_mod.mod_b.D = main_mod.mod_b.create_d(3)
    
    # Use the variables to avoid ruff warnings
    assert isinstance(c, main_mod.mod_a.C)
    assert isinstance(d, main_mod.mod_b.D)
    
    # Return values should be properly typed
    x: int = main_mod.int.dummy_int_fun(5)
    assert x == 5