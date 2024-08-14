from mixed import main_mod


def test_main_mod():
    a = main_mod.create_a(1)
    a.show_x()

    b = main_mod.create_b(1)
    b.show_x()


def test_sub_mod():
    c = main_mod.sub_mod.create_c(1)
    c.show_x()
