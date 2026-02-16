"""Test cases for time crate type support in pyo3-stub-gen"""
import datetime

from pure import (
    get_date,
    get_time,
    get_duration,
    get_primitive_datetime,
    get_offset_datetime,
    get_utc_offset,
    add_duration_to_date,
    time_difference,
)


def test_get_date():
    """Test time::Date to datetime.date conversion"""
    date = get_date(2024, 1, 15)
    assert isinstance(date, datetime.date)
    assert date.year == 2024
    assert date.month == 1
    assert date.day == 15


def test_get_time():
    """Test time::Time to datetime.time conversion"""
    time = get_time(14, 30, 45)
    assert isinstance(time, datetime.time)
    assert time.hour == 14
    assert time.minute == 30
    assert time.second == 45


def test_get_duration():
    """Test time::Duration to datetime.timedelta conversion"""
    duration = get_duration(3600)
    assert isinstance(duration, datetime.timedelta)
    assert duration.total_seconds() == 3600


def test_get_primitive_datetime():
    """Test time::PrimitiveDateTime to datetime.datetime conversion"""
    dt = get_primitive_datetime(2024, 6, 15, 10, 30, 0)
    assert isinstance(dt, datetime.datetime)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0


def test_get_offset_datetime():
    """Test time::OffsetDateTime to datetime.datetime conversion"""
    dt = get_offset_datetime(2024, 6, 15, 10, 30, 0, 9)  # UTC+9
    assert isinstance(dt, datetime.datetime)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_get_utc_offset():
    """Test time::UtcOffset to datetime.tzinfo conversion"""
    offset = get_utc_offset(9)  # UTC+9
    assert isinstance(offset, datetime.tzinfo)


def test_add_duration_to_date():
    """Test adding duration to a date"""
    date = datetime.date(2024, 1, 1)
    duration = datetime.timedelta(days=10)
    result = add_duration_to_date(date, duration)
    assert isinstance(result, datetime.date)
    assert result == datetime.date(2024, 1, 11)


def test_time_difference():
    """Test calculating time difference"""
    time1 = datetime.time(14, 30, 0)
    time2 = datetime.time(10, 0, 0)
    diff = time_difference(time1, time2)
    assert isinstance(diff, datetime.timedelta)
    # time1 - time2 = 4 hours 30 minutes = 16200 seconds
    assert diff.total_seconds() == 16200
