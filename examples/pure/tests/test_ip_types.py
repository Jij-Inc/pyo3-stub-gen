"""Test cases for std::net IP address type support in pyo3-stub-gen

These tests verify that PyStubType generates correct type mappings for
``std::net::Ipv4Addr`` / ``Ipv6Addr`` / ``IpAddr``.
The type annotations on variables are checked by pyright against the generated stubs.
"""
import ipaddress

from pure import (
    ipv4_localhost,
    ipv6_localhost,
    parse_ip,
    is_loopback,
)


def test_ipv4_localhost() -> None:
    """Test std::net::Ipv4Addr to ipaddress.IPv4Address conversion"""
    addr: ipaddress.IPv4Address = ipv4_localhost()
    assert addr == ipaddress.IPv4Address("127.0.0.1")


def test_ipv6_localhost() -> None:
    """Test std::net::Ipv6Addr to ipaddress.IPv6Address conversion"""
    addr: ipaddress.IPv6Address = ipv6_localhost()
    assert addr == ipaddress.IPv6Address("::1")


def test_parse_ip_v4() -> None:
    """Test std::net::IpAddr return type accepts an IPv4 result"""
    addr: ipaddress.IPv4Address | ipaddress.IPv6Address = parse_ip("192.168.0.1")
    assert isinstance(addr, ipaddress.IPv4Address)
    assert addr == ipaddress.IPv4Address("192.168.0.1")


def test_parse_ip_v6() -> None:
    """Test std::net::IpAddr return type accepts an IPv6 result"""
    addr: ipaddress.IPv4Address | ipaddress.IPv6Address = parse_ip("2001:db8::1")
    assert isinstance(addr, ipaddress.IPv6Address)
    assert addr == ipaddress.IPv6Address("2001:db8::1")


def test_is_loopback_v4() -> None:
    """Test std::net::IpAddr argument accepts an IPv4 address"""
    assert is_loopback(ipaddress.IPv4Address("127.0.0.1")) is True
    assert is_loopback(ipaddress.IPv4Address("8.8.8.8")) is False


def test_is_loopback_v6() -> None:
    """Test std::net::IpAddr argument accepts an IPv6 address"""
    assert is_loopback(ipaddress.IPv6Address("::1")) is True
    assert is_loopback(ipaddress.IPv6Address("2001:db8::1")) is False
