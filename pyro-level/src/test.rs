use bytes::{BufMut, Bytes, BytesMut};
use util::{Deserialize, Serialize, Vector, Vector3b};

use crate::{
    biome::Biome3d, database::RawDatabase, DatabaseKey, Dimension, KeyData,
    SubChunk, BIOME_DATA, LOCAL_PLAYER, MOB_EVENTS, OVERWORLD, SCHEDULER,
    SCOREBOARD,
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
            // let x = i32::from_le_bytes((&key[0..4]).try_into().unwrap());
            // let z = i32::from_le_bytes((&key[4..8]).try_into().unwrap());
            // let y = *key.last().unwrap() as i8;
            //
            // println!("{x} {y} {z}");

            let data = Bytes::copy_from_slice(raw_ref.value().as_ref());
            let subchunk = SubChunk::deserialize(data).unwrap();
            let layer = subchunk.layer(0).unwrap();
            let mut layer_iter = layer.iter();

            for block in layer_iter {
                println!("block {block:?}");
            }
        }
    }

    // let mut buffer = BytesMut::new();
    // DatabaseKey {
    //     x: -2,
    //     z: -1,
    //     data: KeyData::SubChunk {
    //         index: 6
    //     },
    //     dimension: Dimension::Overworld
    // }.serialize(&mut buffer);
    //
    // let data = db.get_raw_key(buffer).unwrap();
    // let subchunk = SubChunk::deserialize(data).unwrap();
    // let block = subchunk.get(Vector::from([0, 0, 0])).unwrap();
    // dbg!(block);
}
