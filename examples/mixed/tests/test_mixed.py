from mixed import main_mod


def test_main_mod():
    a = main_mod.create_a(1)
    a.show_x()

    b = main_mod.create_b(1)
    b.show_x()
