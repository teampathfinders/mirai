use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use base64::Engine;
use flate2::read::DeflateDecoder;

use util::{Result, Serialize};
use util::bytes::MutableBuffer;

use crate::network::{Frame, OrderChannel};
use crate::network::Header;
use crate::network::instance::{IPV4_LOCAL_ADDR, IPV6_LOCAL_ADDR};

#[test]
fn read_write_header() {
    let header = Header {
        id: 129,
        sender_subclient: 3,
        target_subclient: 2,
    };

    let mut buffer = MutableBuffer::new();
    header.serialize(&mut buffer).unwrap();

    assert_eq!(Header::deserialize(&mut buffer.snapshot()).unwrap(), header);
}

#[test]
fn order_channel() {
    let mut channel = OrderChannel::new();

    let mut test_frame = Frame::default();
    test_frame.order_index = 0;
    assert!(channel.insert(test_frame).is_some());

    let mut test_frame = Frame::default();
    test_frame.order_index = 2;
    assert!(channel.insert(test_frame).is_none());

    let mut test_frame = Frame::default();
    test_frame.order_index = 1;
    let output = channel.insert(test_frame).unwrap();

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].order_index, 1);
    assert_eq!(output[1].order_index, 2);
}
