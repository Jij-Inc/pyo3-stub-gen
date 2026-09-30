"""Test Jiff runtime conversions and generated type annotations."""

import datetime
import zoneinfo

from pure import (
    jiff_date_round_trip,
    jiff_datetime_round_trip,
    jiff_iso_week_date_round_trip,
    jiff_offset_round_trip,
    jiff_signed_duration_round_trip,
    jiff_span_to_signed_duration,
    jiff_time_round_trip,
    jiff_time_zone_round_trip,
    jiff_timestamp_round_trip,
    jiff_zoned_round_trip,
)


def test_jiff_timestamp_round_trip() -> None:
    value = datetime.datetime(
        2024,
        2,
        29,
        23,
        45,
        12,
        345678,
        tzinfo=datetime.timezone.utc,
    )
    result: datetime.datetime = jiff_timestamp_round_trip(value)

    assert result == value
    assert result.tzinfo == datetime.timezone.utc


def test_jiff_zoned_round_trip() -> None:
    zone = datetime.timezone(datetime.timedelta(hours=9, minutes=30))
    value = datetime.datetime(2025, 3, 14, 8, 9, 10, 111213, tzinfo=zone)
    result: datetime.datetime = jiff_zoned_round_trip(value)

    assert result == value
    assert result.tzinfo == zone


def test_jiff_datetime_round_trip() -> None:
    value = datetime.datetime(2026, 4, 17, 12, 34, 56, 789012)  # noqa: DTZ001
    result: datetime.datetime = jiff_datetime_round_trip(value)

    assert result == value
    assert result.tzinfo is None


def test_jiff_date_round_trip() -> None:
    value = datetime.date(2027, 5, 19)
    result: datetime.date = jiff_date_round_trip(value)

    assert result == value


def test_jiff_time_round_trip() -> None:
    value = datetime.time(6, 7, 8, 901234)
    result: datetime.time = jiff_time_round_trip(value)

    assert result == value


def test_jiff_iso_week_date_round_trip() -> None:
    value = datetime.date(2028, 1, 3)
    result: datetime.date = jiff_iso_week_date_round_trip(value)

    assert result == value


def test_jiff_offset_round_trip() -> None:
    value: datetime.tzinfo = datetime.timezone(
        datetime.timedelta(hours=-5, minutes=-30),
    )
    result: datetime.tzinfo = jiff_offset_round_trip(value)

    assert result == value
    assert result.utcoffset(None) == datetime.timedelta(hours=-5, minutes=-30)


def test_jiff_time_zone_round_trip() -> None:
    value: datetime.tzinfo = zoneinfo.ZoneInfo("Europe/London")
    result: datetime.tzinfo = jiff_time_zone_round_trip(value)

    assert result == value
    assert isinstance(result, zoneinfo.ZoneInfo)
    assert result.key == "Europe/London"


def test_jiff_signed_duration_round_trip() -> None:
    value = datetime.timedelta(days=-2, seconds=12345, microseconds=678901)
    result: datetime.timedelta = jiff_signed_duration_round_trip(value)

    assert result == value


def test_jiff_span_to_signed_duration() -> None:
    value = datetime.timedelta(days=3, seconds=42, microseconds=123456)
    result: datetime.timedelta = jiff_span_to_signed_duration(value)

    assert result == value
