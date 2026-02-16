//! Test cases for `chrono` crate type support in pyo3-stub-gen

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

/// Returns a chrono::NaiveDate from year, month, day
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_naive_date(year: i32, month: u32, day: u32) -> PyResult<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid date"))
}

/// Returns a chrono::NaiveTime from hour, minute, second
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_naive_time(hour: u32, minute: u32, second: u32) -> PyResult<NaiveTime> {
    NaiveTime::from_hms_opt(hour, minute, second)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid time"))
}

/// Returns a chrono::NaiveDateTime from date and time components
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_naive_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> PyResult<NaiveDateTime> {
    let date = get_naive_date(year, month, day)?;
    let time = get_naive_time(hour, minute, second)?;
    Ok(NaiveDateTime::new(date, time))
}

/// Returns a chrono::DateTime<Utc> from components
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_datetime_utc(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> PyResult<DateTime<Utc>> {
    use chrono::TimeZone;
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid datetime"))
}

/// Returns a chrono::DateTime<FixedOffset> from components with offset
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_datetime_fixed_offset(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    offset_hours: i32,
) -> PyResult<DateTime<FixedOffset>> {
    use chrono::TimeZone;
    let offset = FixedOffset::east_opt(offset_hours * 3600)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid offset"))?;
    offset
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid datetime"))
}

/// Returns a chrono::Duration from seconds
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_chrono_duration(seconds: i64) -> PyResult<Duration> {
    Duration::try_seconds(seconds)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid duration"))
}

/// Returns a chrono::FixedOffset from hours
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_fixed_offset(hours: i32) -> PyResult<FixedOffset> {
    FixedOffset::east_opt(hours * 3600)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid offset"))
}

/// Returns chrono::Utc timezone
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_utc() -> Utc {
    Utc
}

/// Add duration to a NaiveDate
#[gen_stub_pyfunction]
#[pyfunction]
pub fn add_chrono_duration_to_date(date: NaiveDate, duration: Duration) -> NaiveDate {
    date + duration
}

/// Calculate the difference between two NaiveTimes as duration
#[gen_stub_pyfunction]
#[pyfunction]
pub fn naive_time_difference(time1: NaiveTime, time2: NaiveTime) -> Duration {
    time1 - time2
}
