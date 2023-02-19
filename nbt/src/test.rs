use bytes::{Bytes, BytesMut};

use crate::{RefTag, Tag, Value};
use std::collections::HashMap;

const BIGTEST_NBT: &[u8] = include_bytes!("../test/bigtest.nbt");
const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test/hello_world.nbt");
const PLAYER_NAN_VALUE_NBT: &[u8] =
    include_bytes!("../test/player_nan_value.nbt");

#[test]
fn hello_world_write_nbt() {
    let tag = RefTag {
        name: "hello world",
        value: &Value::Compound(HashMap::from([(
            "name".to_owned(),
            Value::String("Bananrama".to_owned()),
        )])),
    };

    let mut encoded = BytesMut::new();
    tag.write_be(&mut encoded);
    assert_eq!(encoded, HELLO_WORLD_NBT);
}

#[test]
fn bigtest_nbt() {
    crate::read_be(&mut Bytes::from(BIGTEST_NBT)).unwrap();
}

#[test]
fn hello_world_nbt() {
    let decoded = crate::read_be(&mut Bytes::from(HELLO_WORLD_NBT)).unwrap();
    println!("{decoded:?}");

    assert_eq!(
        decoded,
        Tag {
            name: "hello world".to_owned(),
            value: Value::Compound(HashMap::from([(
                "name".to_owned(),
                Value::String("Bananrama".to_owned())
            )]))
        }
    );
}

#[test]
fn player_nan_value_nbt() {
    crate::read_be(&mut Bytes::from(PLAYER_NAN_VALUE_NBT)).unwrap();
}
