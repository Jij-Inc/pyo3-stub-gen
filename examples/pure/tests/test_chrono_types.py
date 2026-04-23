"""Test cases for chrono crate type support in pyo3-stub-gen

These tests verify that PyStubType generates correct type mappings for chrono types.
The type annotations on variables are checked by pyright against the generated stubs.
"""
import datetime

from pure import (
    get_naive_date,
    get_naive_time,
    get_naive_datetime,
    get_datetime_utc,
    get_datetime_fixed_offset,
    get_chrono_duration,
    get_fixed_offset,
    get_utc,
    add_chrono_duration_to_date,
    naive_time_difference,
)


def test_get_naive_date() -> None:
    """Test chrono::NaiveDate to datetime.date conversion"""
    date: datetime.date = get_naive_date(2024, 1, 15)
    assert date.year == 2024
    assert date.month == 1
    assert date.day == 15


def test_get_naive_time() -> None:
    """Test chrono::NaiveTime to datetime.time conversion"""
    time: datetime.time = get_naive_time(14, 30, 45)
    assert time.hour == 14
    assert time.minute == 30
    assert time.second == 45


def test_get_naive_datetime() -> None:
    """Test chrono::NaiveDateTime to datetime.datetime conversion"""
    dt: datetime.datetime = get_naive_datetime(2024, 6, 15, 10, 30, 0)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0


def test_get_datetime_utc() -> None:
    """Test chrono::DateTime<Utc> to datetime.datetime conversion"""
    dt: datetime.datetime = get_datetime_utc(2024, 6, 15, 10, 30, 0)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_get_datetime_fixed_offset() -> None:
    """Test chrono::DateTime<FixedOffset> to datetime.datetime conversion"""
    dt: datetime.datetime = get_datetime_fixed_offset(2024, 6, 15, 10, 30, 0, 9)  # UTC+9
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_get_chrono_duration() -> None:
    """Test chrono::Duration to datetime.timedelta conversion"""
    duration: datetime.timedelta = get_chrono_duration(3600)
    assert duration.total_seconds() == 3600


def test_get_fixed_offset() -> None:
    """Test chrono::FixedOffset to datetime.tzinfo conversion"""
    offset: datetime.tzinfo = get_fixed_offset(9)  # UTC+9
    assert offset is not None


def test_get_utc() -> None:
    """Test chrono::Utc to datetime.tzinfo conversion"""
    utc: datetime.tzinfo = get_utc()
    assert utc is not None


def test_add_chrono_duration_to_date() -> None:
    """Test adding chrono duration to a date"""
    date: datetime.date = datetime.date(2024, 1, 1)
    duration: datetime.timedelta = datetime.timedelta(days=10)
    result: datetime.date = add_chrono_duration_to_date(date, duration)
    assert result == datetime.date(2024, 1, 11)


def test_naive_time_difference() -> None:
    """Test calculating time difference with chrono"""
    time1: datetime.time = datetime.time(14, 30, 0)
    time2: datetime.time = datetime.time(10, 0, 0)
    diff: datetime.timedelta = naive_time_difference(time1, time2)
    # time1 - time2 = 4 hours 30 minutes = 16200 seconds
    assert diff.total_seconds() == 16200
