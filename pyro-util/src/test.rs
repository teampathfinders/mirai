use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::bytes::LazyBuffer;
use crate::{Result, Vector};

#[test]
fn vector_types() {
    let mut vec1 = Vector::from([1]);
    assert_eq!(vec1.x, 1);

    vec1.components_mut()[0] = 2;
    assert_eq!(vec1.x, 2);
    vec1.x = 3;
    assert_eq!(vec1.x, 3);

    let mut vec2 = Vector::from([1, 2]);
    assert_eq!(vec2.x, 1);
    assert_eq!(vec2.y, 2);

    vec2.components_mut()[1] = 3;
    assert_eq!(vec2.y, 3);
    vec2.y = 4;
    assert_eq!(vec2.y, 4);

    let mut vec3 = Vector::from([1, 2, 3]);
    assert_eq!(vec3.x, 1);
    assert_eq!(vec3.y, 2);
    assert_eq!(vec3.z, 3);

    vec3.components_mut()[2] = 4;
    assert_eq!(vec3.z, 4);
    vec3.z = 5;
    assert_eq!(vec3.z, 5);

    let mut vec4 = Vector::from([1, 2, 3, 4]);
    assert_eq!(vec4.x, 1);
    assert_eq!(vec4.y, 2);
    assert_eq!(vec4.z, 3);
    assert_eq!(vec4.w, 4);

    vec4.components_mut()[3] = 5;
    assert_eq!(vec4.w, 5);
    vec4.w = 6;
    assert_eq!(vec4.w, 6);
}

#[test]
fn read_write_string() {
    let mut buffer = LazyBuffer::new();
    buffer.put_string("Hello World!");

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_string().unwrap(), "Hello World!");
}

#[test]
fn read_write_raknet_string() {
    let mut buffer = LazyBuffer::new();
    buffer.put_raknet_string("Hello World!");

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_raknet_string(), "Hello World!");
}

#[test]
fn read_write_var_u32() {
    let mut buffer = LazyBuffer::new();
    buffer.put_var_u32(45);
    buffer.put_var_u32(2769);
    buffer.put_var_u32(105356);
    buffer.put_var_u32(359745976);

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_var_u32().unwrap(), 45);
    assert_eq!(buffer.get_var_u32().unwrap(), 2769);
    assert_eq!(buffer.get_var_u32().unwrap(), 105356);
    assert_eq!(buffer.get_var_u32().unwrap(), 359745976);
}

#[test]
fn read_write_var_i32() {
    let mut buffer = LazyBuffer::new();
    buffer.put_var_i32(45);
    buffer.put_var_i32(-2769);
    buffer.put_var_i32(105356);
    buffer.put_var_i32(-3597459);

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_var_i32().unwrap(), 45);
    assert_eq!(buffer.get_var_i32().unwrap(), -2769);
    assert_eq!(buffer.get_var_i32().unwrap(), 105356);
    assert_eq!(buffer.get_var_i32().unwrap(), -3597459);
}

#[test]
fn read_write_var_u64() {
    let mut buffer = LazyBuffer::new();
    buffer.put_var_u64(45);
    buffer.put_var_u64(2769);
    buffer.put_var_u64(105356);
    buffer.put_var_u64(359745976);
    buffer.put_var_u64(35974597639766);

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_var_u64().unwrap(), 45);
    assert_eq!(buffer.get_var_u64().unwrap(), 2769);
    assert_eq!(buffer.get_var_u64().unwrap(), 105356);
    assert_eq!(buffer.get_var_u64().unwrap(), 359745976);
    assert_eq!(buffer.get_var_u64().unwrap(), 35974597639766);
}

#[test]
fn read_write_var_i64() {
    let mut buffer = LazyBuffer::new();
    buffer.put_var_i32(45);
    buffer.put_var_i32(-2769);
    buffer.put_var_i32(105356);
    buffer.put_var_i32(-3597459);

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_var_i32().unwrap(), 45);
    assert_eq!(buffer.get_var_i32().unwrap(), -2769);
    assert_eq!(buffer.get_var_i32().unwrap(), 105356);
    assert_eq!(buffer.get_var_i32().unwrap(), -3597459);
}

#[test]
fn read_write_u24_le() {
    let mut buffer = LazyBuffer::new();
    buffer.put_u24_le(125); // Test first byte only
    buffer.put_u24_le(50250); // Test first two bytes
    buffer.put_u24_le(1097359); // Test all bytes

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_u24_le(), 125);
    assert_eq!(buffer.get_u24_le(), 50250);
    assert_eq!(buffer.get_u24_le(), 1097359);
}

#[test]
fn read_write_addr() -> Result<()> {
    let ipv4_test = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), 19132);
    let ipv6_test = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 19133);

    let mut buffer = LazyBuffer::new();
    buffer.put_addr(ipv4_test); // Test IPv4
    buffer.put_addr(ipv6_test); // Test IPv6

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_addr()?, ipv4_test);
    assert_eq!(buffer.get_addr()?, ipv6_test);
    Ok(())
}
