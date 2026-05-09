//! Test cases for `std::net` IP address type support in pyo3-stub-gen
//!
//! Note: `Ipv4Addr` and `Ipv6Addr` are only `IntoPyObject` in PyO3, not `FromPyObject`.
//! Only `IpAddr` is `FromPyObject`, so input arguments use `IpAddr`.

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Returns the IPv4 loopback address (127.0.0.1).
#[gen_stub_pyfunction]
#[pyfunction]
pub fn ipv4_localhost() -> Ipv4Addr {
    Ipv4Addr::LOCALHOST
}

/// Returns the IPv6 loopback address (::1).
#[gen_stub_pyfunction]
#[pyfunction]
pub fn ipv6_localhost() -> Ipv6Addr {
    Ipv6Addr::LOCALHOST
}

/// Parses a string into an IpAddr (either IPv4 or IPv6).
#[gen_stub_pyfunction]
#[pyfunction]
pub fn parse_ip(s: &str) -> PyResult<IpAddr> {
    s.parse::<IpAddr>()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid IP: {}", e)))
}

/// Returns whether the given IP address is a loopback address.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn is_loopback(addr: IpAddr) -> bool {
    addr.is_loopback()
}
