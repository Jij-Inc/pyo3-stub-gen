from mixed_sub_multiple import main_mod


def test_main_mod():
    main_mod.greet_main()


def test_sub_mod_a():
    main_mod.mod_a.greet_a()


def test_sub_mod_b():
    main_mod.mod_b.greet_b()
