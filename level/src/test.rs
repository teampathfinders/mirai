use bytes::{BytesMut, Bytes, BufMut};
use common::{Deserialize, Serialize, Vector3b};

use crate::{
    database::RawDatabase, DatabaseKey, KeyData, Dimension, SubChunk, biome::Biome3d, LOCAL_PLAYER, BIOME_DATA, OVERWORLD, SCHEDULER, MOB_EVENTS, SCOREBOARD,
};

// digp [x] [z] [?dimension]
// contains two int32
// can pack multiple into one
// points to "actorprefix" + digp data

// palette: [Compound({"states": Compound({"pillar_axis": String("y")}), "version": Int(17959425), "name": String("minecraft:deepslate")}), Compound({"states": Compound({"stone_type": String("stone")}), "version": Int(17959425), "name": String("minecraft:stone")}), Compound({"states": Compound({}), "name": String("minecraft:iron_ore"), "version": Int(17959425)}), Compound({"name": String("minecraft:gravel"), "states": Compound({}), "version": Int(17959425)}), Compound({"states": Compound({}), "name": String("minecraft:deepslate_iron_ore"), "version": Int(17959425)}), Compound({"states": Compound({"stone_type": String("diorite")}), "version": Int(17959425), "name": String("minecraft:stone")}), Compound({"name": String("minecraft:dirt"), "states": Compound({"dirt_type": String("normal")}), "version": Int(17959425)}), Compound({"states": Compound({}), "version": Int(17959425), "name": String("minecraft:deepslate_redstone_ore")}), Compound({"version": Int(17959425), "states": Compound({}), "name": String("minecraft:deepslate_copper_ore")}), Compound({"name": String("minecraft:copper_ore"), "version": Int(17959425), "states": Compound({})}), Compound({"states": Compound({}), "name": String("minecraft:deepslate_lapis_ore"), "version": Int(17959425)}), Compound({"version": Int(17959425), "name": String("minecraft:stone"), "states": Compound({"stone_type": String("granite")})}), Compound({"states": Compound({}), "version": Int(17959425), "name": String("minecraft:lapis_ore")}), Compound({"version": Int(17959425), "name": String("minecraft:redstone_ore"), "states": Compound({})}), Compound({"version": Int(17959425), "states": Compound({"stone_type": String("andesite")}), "name": String("minecraft:stone")}), Compound({"version": Int(17959425), "name": String("minecraft:air"), "states": Compound({})})] }]

#[test]
fn database_test() {
    let db = RawDatabase::new("test/db").unwrap();
    let mut iter = db.iter();

    for raw_ref in iter {
        let key = raw_ref.key();
        if key[key.len() - 2] == 0x2f {
            let data = Bytes::copy_from_slice(raw_ref.value().as_ref());
            let subchunk = SubChunk::deserialize(data).unwrap();
            println!("{subchunk:?}");

            break
        }
    }

    // let mut key = BytesMut::from("digp".as_bytes());
    // key.put_i32_le(0);
    // key.put_i32_le(0);

    // let data = db.get_raw_key(key).unwrap();
    // // println!("{data:?}");

    // let mut key2 = BytesMut::from("actorprefix");
    // key2.put(data);

    // let mut data2 = db.get_raw_key(key2).unwrap();
    // println!("{:?}", data2.as_ref());

    // let nbt = nbt::deserialize_le(&mut data2).unwrap();
    // println!("{nbt:?}");

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