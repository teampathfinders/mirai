use std::collections::HashMap;
use std::io::Write;
use std::num::FpCategory::Nan;
use bytes::{BufMut, BytesMut};
use crate::{OwnedTag, RefTag, Value};

const BIGTEST_NBT: &[u8] = include_bytes!("../test/bigtest.nbt");
const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test/hello_world.nbt");
const PLAYER_NAN_VALUE_NBT: &[u8] = include_bytes!("../test/player_nan_value.nbt");

#[test]
fn hello_world_write_nbt() {
    let tag = RefTag {
        name: "hello world",
        value: &Value::Compound(HashMap::from([
            ("name".to_owned(), Value::String("Bananrama".to_owned()))
        ]))
    };

    let encoded = tag.encode_be();
    assert_eq!(encoded, HELLO_WORLD_NBT);
}

#[test]
fn bigtest_nbt() {
    OwnedTag::decode_be(BIGTEST_NBT).unwrap();
}

#[test]
fn hello_world_nbt() {
    let decoded = OwnedTag::decode_be(HELLO_WORLD_NBT).unwrap();
    println!("{decoded:?}");

    assert_eq!(decoded, OwnedTag {
        name: "hello world".to_owned(),
        value: Value::Compound(HashMap::from(
            [
                ("name".to_owned(), Value::String("Bananrama".to_owned()))
            ]
        ))
    });
}

#[test]
fn player_nan_value_nbt() {
    OwnedTag::decode_be(PLAYER_NAN_VALUE_NBT).unwrap();
}