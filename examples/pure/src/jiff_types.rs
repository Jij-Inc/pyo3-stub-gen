//! Test cases for `jiff` crate type support in pyo3-stub-gen

use jiff_02::{
    civil::{Date, DateTime, ISOWeekDate, Time},
    tz::{Offset, TimeZone},
    SignedDuration, Span, Timestamp, Zoned,
};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

/// Round-trips a Jiff timestamp through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_timestamp_round_trip(value: Timestamp) -> Timestamp {
    value
}

/// Round-trips a Jiff zoned datetime through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_zoned_round_trip(value: Zoned) -> Zoned {
    value
}

/// Round-trips a Jiff civil datetime through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_datetime_round_trip(value: DateTime) -> DateTime {
    value
}

/// Round-trips a Jiff civil date through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_date_round_trip(value: Date) -> Date {
    value
}

/// Round-trips a Jiff civil time through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_time_round_trip(value: Time) -> Time {
    value
}

/// Round-trips a Jiff ISO week date through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_iso_week_date_round_trip(value: ISOWeekDate) -> ISOWeekDate {
    value
}

/// Round-trips a Jiff fixed offset through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_offset_round_trip(value: Offset) -> Offset {
    value
}

/// Round-trips a Jiff time zone through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_time_zone_round_trip(value: TimeZone) -> TimeZone {
    value
}

/// Round-trips a Jiff signed duration through Python.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_signed_duration_round_trip(value: SignedDuration) -> SignedDuration {
    value
}

/// Converts a Python duration accepted as a Jiff span into a signed duration.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn jiff_span_to_signed_duration(value: Span) -> PyResult<SignedDuration> {
    value.try_into().map_err(Into::into)
}
