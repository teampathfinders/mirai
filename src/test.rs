use std::io::Read;
use std::net::{IpAddr, SocketAddr};

use bytes::{Buf, BufMut, BytesMut};
use flate2::read::DeflateDecoder;
use tokio::net::windows::named_pipe::PipeMode::Byte;

use crate::instance::{IPV4_LOCAL_ADDR, IPV6_LOCAL_ADDR};
use crate::network::raknet::{Frame, Header};
use crate::network::session::OrderChannel;
use crate::util::{AsyncDeque, ReadExtensions, WriteExtensions};

#[test]
fn read_write_header() {
    let header = Header {
        id: 129,
        sender_subclient: 3,
        target_subclient: 2,
    };

    let mut buffer = header.encode();
    assert_eq!(Header::decode(&mut buffer).unwrap(), header);
}

#[test]
fn read_write_string() {
    let mut buffer = BytesMut::new();
    buffer.put_string("Hello World!");

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_string().unwrap(), "Hello World!");
}

#[test]
fn read_write_raknet_string() {
    let mut buffer = BytesMut::new();
    buffer.put_raknet_string("Hello World!");

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_raknet_string(), "Hello World!");
}

#[test]
fn read_write_var_u32() {
    let mut buffer = BytesMut::new();
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
    let mut buffer = BytesMut::new();
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
    let mut buffer = BytesMut::new();
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
    let mut buffer = BytesMut::new();
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
fn read_write_addr() -> VResult<()> {
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

#[test]
fn order_channel() {
    let mut test_frame = Frame::default();
    let mut channel = OrderChannel::new();

    test_frame.order_index = 0;
    assert!(channel.insert(test_frame.clone()).is_some());

    test_frame.order_index = 2;
    assert!(channel.insert(test_frame.clone()).is_none());

    test_frame.order_index = 1;
    let output = channel.insert(test_frame).unwrap();

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].order_index, 1);
    assert_eq!(output[1].order_index, 2);
}
