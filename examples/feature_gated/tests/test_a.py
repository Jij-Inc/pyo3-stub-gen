import feature_gated


def test_a():
    a = feature_gated.A()
    assert a.x == 0
    assert a.get_y() == 0
