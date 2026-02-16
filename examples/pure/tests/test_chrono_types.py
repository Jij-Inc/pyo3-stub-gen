"""Test cases for chrono crate type support in pyo3-stub-gen"""
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


def test_get_naive_date():
    """Test chrono::NaiveDate to datetime.date conversion"""
    date = get_naive_date(2024, 1, 15)
    assert isinstance(date, datetime.date)
    assert date.year == 2024
    assert date.month == 1
    assert date.day == 15


def test_get_naive_time():
    """Test chrono::NaiveTime to datetime.time conversion"""
    time = get_naive_time(14, 30, 45)
    assert isinstance(time, datetime.time)
    assert time.hour == 14
    assert time.minute == 30
    assert time.second == 45


def test_get_naive_datetime():
    """Test chrono::NaiveDateTime to datetime.datetime conversion"""
    dt = get_naive_datetime(2024, 6, 15, 10, 30, 0)
    assert isinstance(dt, datetime.datetime)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0


def test_get_datetime_utc():
    """Test chrono::DateTime<Utc> to datetime.datetime conversion"""
    dt = get_datetime_utc(2024, 6, 15, 10, 30, 0)
    assert isinstance(dt, datetime.datetime)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_get_datetime_fixed_offset():
    """Test chrono::DateTime<FixedOffset> to datetime.datetime conversion"""
    dt = get_datetime_fixed_offset(2024, 6, 15, 10, 30, 0, 9)  # UTC+9
    assert isinstance(dt, datetime.datetime)
    assert dt.year == 2024
    assert dt.month == 6
    assert dt.day == 15
    assert dt.hour == 10
    assert dt.minute == 30
    assert dt.second == 0
    assert dt.tzinfo is not None


def test_get_chrono_duration():
    """Test chrono::Duration to datetime.timedelta conversion"""
    duration = get_chrono_duration(3600)
    assert isinstance(duration, datetime.timedelta)
    assert duration.total_seconds() == 3600


def test_get_fixed_offset():
    """Test chrono::FixedOffset to datetime.tzinfo conversion"""
    offset = get_fixed_offset(9)  # UTC+9
    assert isinstance(offset, datetime.tzinfo)


def test_get_utc():
    """Test chrono::Utc to datetime.tzinfo conversion"""
    utc = get_utc()
    assert isinstance(utc, datetime.tzinfo)


def test_add_chrono_duration_to_date():
    """Test adding chrono duration to a date"""
    date = datetime.date(2024, 1, 1)
    duration = datetime.timedelta(days=10)
    result = add_chrono_duration_to_date(date, duration)
    assert isinstance(result, datetime.date)
    assert result == datetime.date(2024, 1, 11)


def test_naive_time_difference():
    """Test calculating time difference with chrono"""
    time1 = datetime.time(14, 30, 0)
    time2 = datetime.time(10, 0, 0)
    diff = naive_time_difference(time1, time2)
    assert isinstance(diff, datetime.timedelta)
    # time1 - time2 = 4 hours 30 minutes = 16200 seconds
    assert diff.total_seconds() == 16200
