from mixed_sub_import_type import main_mod


def test_main_mod():
    a = main_mod.create_a(1)
    a.show_x()

    b = main_mod.create_b(1)
    b.show_x()


def test_sub_mod():
    a = main_mod.create_a(1)
    b = main_mod.create_b(1)

    c = main_mod.sub_mod.create_c(a, b)
    c.show_x()
