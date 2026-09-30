"""Check UUID conversions and the generated stubs with Python type checkers."""

import uuid

import pytest
from typing_extensions import assert_type

from pure import echo_uuid


@pytest.mark.parametrize(
    "value",
    [
        uuid.UUID(int=0),
        uuid.UUID("00112233-4455-6677-8899-aabbccddeeff"),
        uuid.UUID(int=(1 << 128) - 1),
    ],
    ids=["nil", "mixed-bytes", "max"],
)
def test_uuid_roundtrip(value: uuid.UUID) -> None:
    result = assert_type(echo_uuid(value), uuid.UUID)
    assert isinstance(result, uuid.UUID)
    assert result == value
