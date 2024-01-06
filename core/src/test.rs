use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use flate2::read::DeflateDecoder;
use proto::base64::Engine;

use util::MutableBuffer;
use util::{Result, Serialize};

use crate::instance::{IPV4_LOCAL_ADDR, IPV6_LOCAL_ADDR};
use crate::raknet::{Frame, OrderChannel};
use proto::bedrock::Header;

#[test]
fn biome_nbt() {
    let biomes: nbt::Value = nbt::from_var_bytes(include_bytes!("../include/biomes.nbt").as_ref()).unwrap().0;
    dbg!(biomes);
}

#[test]
fn header() {
    let header = Header {
        id: 129,
        sender_subclient: 3,
        target_subclient: 2,
    };

    let mut buffer = MutableBuffer::new();
    header.serialize(&mut buffer).unwrap();

    assert_eq!(Header::deserialize(&mut buffer.snapshot()).unwrap(), header);
}
