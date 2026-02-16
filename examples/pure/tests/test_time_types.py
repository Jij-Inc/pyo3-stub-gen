"""Test cases for time crate type support in pyo3-stub-gen

These tests verify that PyStubType generates correct type mappings for time crate types.
The type annotations on variables are checked by pyright against the generated stubs.
"""
import datetime

from pure import (
    get_date,
    get_time,
    get_duration,
    get_primitive_datetime,
    get_offset_datetime,
    get_utc_offset,
    get_utc_datetime,
    add_duration_to_date,
    time_difference,
)


def test_get_date() -> None:
    """Test time::Date to datetime.date conversion"""
    date: datetime.date = get_date(2024, 1, 15)
    assert date.year == 2024
    assert date.month == 1
    assert date.day == 15


def test_get_time() -> None:
    """Test time::Time to datetime.time conversion"""
    time: datetime.time = get_time(14, 30, 45)
    assert time.hour == 14
    assert time.minute == 30
    assert time.second == 45


def test_get_duration() -> None:
    """Test time::Duration to datetime.timedelta conversion"""
    duration: datetime.timedelta = get_duration(3600)
    assert duration.total_seconds() == 3600


def test_get_primitive_datetime() -> None:
    """Test time::PrimitiveDateTime to datetime.datetime conversion"""
    dt: datetime.datetime = get_primitive_datetime(2024, 6, 15, 10, 30, 0)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0


def test_get_offset_datetime() -> None:
    """Test time::OffsetDateTime to datetime.datetime conversion"""
    dt: datetime.datetime = get_offset_datetime(2024, 6, 15, 10, 30, 0, 9)  # UTC+9
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_get_utc_offset() -> None:
    """Test time::UtcOffset to datetime.tzinfo conversion"""
    offset: datetime.tzinfo = get_utc_offset(9)  # UTC+9
    assert offset is not None


def test_get_utc_datetime() -> None:
    """Test time::UtcDateTime to datetime.datetime conversion"""
    dt: datetime.datetime = get_utc_datetime(2024, 6, 15, 10, 30, 0)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_add_duration_to_date() -> None:
    """Test adding duration to a date"""
    date: datetime.date = datetime.date(2024, 1, 1)
    duration: datetime.timedelta = datetime.timedelta(days=10)
    result: datetime.date = add_duration_to_date(date, duration)
    assert result == datetime.date(2024, 1, 11)


def test_time_difference() -> None:
    """Test calculating time difference"""
    time1: datetime.time = datetime.time(14, 30, 0)
    time2: datetime.time = datetime.time(10, 0, 0)
    diff: datetime.timedelta = time_difference(time1, time2)
    # time1 - time2 = 4 hours 30 minutes = 16200 seconds
    assert diff.total_seconds() == 16200
