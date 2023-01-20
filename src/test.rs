use std::net::{IpAddr, SocketAddr};
use bytes::BytesMut;
use crate::error::VexResult;
use crate::services::{IPV4_LOCAL_ADDR, IPV6_LOCAL_ADDR};
use crate::util::{ReadExtensions, WriteExtensions};

#[test]
fn read_write_u24_le() {
    let mut buffer = BytesMut::new();
    buffer.put_u24_le(125); // Test first byte only
    buffer.put_u24_le(50250); // Test first two bytes
    buffer.put_u24_le(1097359); // Test all bytes

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_u24_le(), 125);
    assert_eq!(buffer.get_u24_le(), 50250);
    assert_eq!(buffer.get_u24_le(), 1097359);
}

#[test]
fn read_write_addr() -> VexResult<()> {
    let ipv4_test = SocketAddr::new(IpAddr::V4(IPV4_LOCAL_ADDR), 19132);
    let ipv6_test = SocketAddr::new(IpAddr::V6(IPV6_LOCAL_ADDR), 19133);

    let mut buffer = BytesMut::new();
    buffer.put_addr(ipv4_test); // Test IPv4
    buffer.put_addr(ipv6_test); // Test IPv6

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_addr()?, ipv4_test);
    assert_eq!(buffer.get_addr()?, ipv6_test);
    Ok(())
}