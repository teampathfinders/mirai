use std::ops::Deref;

use bytes::{BytesMut, Bytes, BufMut};
use common::{Deserialize, Serialize, Vector3b};

use crate::{
    database::RawDatabase, DatabaseKey, KeyData, Dimension, SubChunk, biome::Biome3d, LOCAL_PLAYER, BIOME_DATA, OVERWORLD, SCHEDULER, MOB_EVENTS, SCOREBOARD,
};

// digp [x] [z] [?dimension]
// contains two int32
// can pack multiple into one
// points to "actorprefix" + digp data

#[test]
fn database_test() {
    let db = RawDatabase::new("test/db").unwrap();
    let mut key = BytesMut::from("digp".as_bytes());
    key.put_i32_le(0);
    key.put_i32_le(0);

    let data = db.get_raw_key(key).unwrap();
    // println!("{data:?}");

    let mut key2 = BytesMut::from("actorprefix");
    key2.put(data);

    let mut data2 = db.get_raw_key(key2).unwrap();
    println!("{:?}", data2.as_ref());

    let nbt = nbt::deserialize_le(&mut data2).unwrap();
    println!("{nbt:?}");

    // let mut iter = db.iter();

    // for raw_ref in iter {
    //     let key = raw_ref.key();
    //     println!("{:0x?}", String::from_utf8_lossy(key.deref()));
    // }
}

// "actorprefix\0\0\0\u{4}\0\0\0\t"
// "actorprefix\0\0\0\u{4}\0\0\0\n"
// "actorprefix\0\0\0\u{4}\0\0\0\u{b}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{f}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{10}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{11}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{12}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{13}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{14}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{15}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{16}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{17}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{18}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{19}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{1a}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{1c}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{1d}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{1e}"
// "actorprefix\0\0\0\u{4}\0\0\0\u{1f}"
// "actorprefix\0\0\0\u{4}\0\0\0 "
// "actorprefix\0\0\0\u{4}\0\0\0!"
// "actorprefix\0\0\0\u{5}\0\0\0\u{1}"
// "actorprefix\0\0\0\u{5}\0\0\0\u{2}"
// "digp\0\0\0\0\0\0\0\0"
// "digp\0\0\0\0\u{1}\0\0\0"
// "digp\0\0\0\0\u{2}\0\0\0"
// "digp\0\0\0\0����"
// "digp\0\0\0\0����"
// "digp\0\0\0\0����"
// "digp\u{1}\0\0\0\0\0\0\0"
// "digp\u{1}\0\0\0\u{1}\0\0\0"
// "digp\u{1}\0\0\0\u{2}\0\0\0"
// "digp\u{1}\0\0\0����"
// "digp\u{1}\0\0\0����"
// "digp\u{1}\0\0\0����"
// "digp\u{2}\0\0\0\0\0\0\0"
// "digp\u{2}\0\0\0\u{1}\0\0\0"
// "digp\u{2}\0\0\0\u{2}\0\0\0"
// "digp\u{2}\0\0\0\u{4}\0\0\0"
// "digp\u{2}\0\0\0����"
// "digp\u{2}\0\0\0����"
// "digp\u{3}\0\0\0\0\0\0\0"
// "digp\u{3}\0\0\0\u{2}\0\0\0"
// "digp\u{3}\0\0\0\u{4}\0\0\0"
// "digp\u{3}\0\0\0����"
// "digp\u{3}\0\0\0����"
// "digp\u{3}\0\0\0����"