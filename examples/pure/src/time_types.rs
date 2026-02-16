//! Test cases for `time` crate type support in pyo3-stub-gen

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

/// Returns the current date as a time::Date
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_date(year: i32, month: u8, day: u8) -> PyResult<Date> {
    let month = time::Month::try_from(month)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid month: {}", e)))?;
    Date::from_calendar_date(year, month, day)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid date: {}", e)))
}

/// Returns a time::Time from hour, minute, second
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_time(hour: u8, minute: u8, second: u8) -> PyResult<Time> {
    Time::from_hms(hour, minute, second)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid time: {}", e)))
}

/// Returns a time::Duration from seconds
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_duration(seconds: i64) -> Duration {
    Duration::seconds(seconds)
}

/// Returns a time::PrimitiveDateTime from date and time components
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_primitive_datetime(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> PyResult<PrimitiveDateTime> {
    let date = get_date(year, month, day)?;
    let time = get_time(hour, minute, second)?;
    Ok(PrimitiveDateTime::new(date, time))
}

/// Returns a time::OffsetDateTime from components with UTC offset
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_offset_datetime(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    offset_hours: i8,
) -> PyResult<OffsetDateTime> {
    let primitive = get_primitive_datetime(year, month, day, hour, minute, second)?;
    let offset = UtcOffset::from_hms(offset_hours, 0, 0)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid offset: {}", e)))?;
    Ok(primitive.assume_offset(offset))
}

/// Returns a time::UtcOffset from hours
#[gen_stub_pyfunction]
#[pyfunction]
pub fn get_utc_offset(hours: i8) -> PyResult<UtcOffset> {
    UtcOffset::from_hms(hours, 0, 0)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid offset: {}", e)))
}

/// Add duration to a date
#[gen_stub_pyfunction]
#[pyfunction]
pub fn add_duration_to_date(date: Date, duration: Duration) -> Date {
    date + duration
}

/// Calculate the difference between two times as duration
#[gen_stub_pyfunction]
#[pyfunction]
pub fn time_difference(time1: Time, time2: Time) -> Duration {
    time1 - time2
}
