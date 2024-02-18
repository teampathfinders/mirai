use std::sync::Mutex;

use crate::provider::Provider;

// digp [x] [z] [?dimension]
// contains two int32
// can pack multiple into one
// points to "actorprefix" + digp data

// palette: [Compound({"states": Compound({"pillar_axis": String("y")}), "version": Int(17959425), "name": String("minecraft:deepslate")}), Compound({"states": Compound({"stone_type": String("stone")}), "version": Int(17959425), "name": String("minecraft:stone")}), Compound({"states": Compound({}), "name": String("minecraft:iron_ore"), "version": Int(17959425)}), Compound({"name": String("minecraft:gravel"), "states": Compound({}), "version": Int(17959425)}), Compound({"states": Compound({}), "name": String("minecraft:deepslate_iron_ore"), "version": Int(17959425)}), Compound({"states": Compound({"stone_type": String("diorite")}), "version": Int(17959425), "name": String("minecraft:stone")}), Compound({"name": String("minecraft:dirt"), "states": Compound({"dirt_type": String("normal")}), "version": Int(17959425)}), Compound({"states": Compound({}), "version": Int(17959425), "name": String("minecraft:deepslate_redstone_ore")}), Compound({"version": Int(17959425), "states": Compound({}), "name": String("minecraft:deepslate_copper_ore")}), Compound({"name": String("minecraft:copper_ore"), "version": Int(17959425), "states": Compound({})}), Compound({"states": Compound({}), "name": String("minecraft:deepslate_lapis_ore"), "version": Int(17959425)}), Compound({"version": Int(17959425), "name": String("minecraft:stone"), "states": Compound({"stone_type": String("granite")})}), Compound({"states": Compound({}), "version": Int(17959425), "name": String("minecraft:lapis_ore")}), Compound({"version": Int(17959425), "name": String("minecraft:redstone_ore"), "states": Compound({})}), Compound({"version": Int(17959425), "states": Compound({"stone_type": String("andesite")}), "name": String("minecraft:stone")}), Compound({"version": Int(17959425), "name": String("minecraft:air"), "states": Compound({})})] }]

static LOCK: Mutex<()> = Mutex::new(());

#[test]
fn level_settings() {
    let _lock = LOCK.lock().unwrap();
    let provider = unsafe { Provider::open("../../resources/level").unwrap() };
    let _settings = provider.settings().unwrap();
}
//
// #[test]
// fn chunk_version() {
//     let _lock = LOCK.lock().unwrap();
//     let provider = unsafe {
//         Provider::open("test").unwrap()
//     };
//     let version = provider.get_version(Vector::from([0, 0]), Dimension::Overworld).unwrap();
//     assert_eq!(version, Some(40));
// }
//
// #[test]
// fn key_not_found() {
//     let _lock = LOCK.lock().unwrap();
//     let provider = unsafe {
//         Provider::open("test").unwrap()
//     };
//     let biome = provider.get_biomes(Vector::from([1290712972, 29372937]), Dimension::Overworld).unwrap();
//     assert_eq!(biome, None);
// }
//
// #[test]
// fn biomes() {
//     let _lock = LOCK.lock().unwrap();
//     let database = unsafe {
//         Database::open("test/db").unwrap()
//     };
//     let iter = database.iter();
//
//     for kv in iter {
//         let key = kv.key();
//         if *key.last().unwrap() == KeyType::Biome3d.discriminant() {
//             let biome = Biomes::deserialize(&*kv.value()).unwrap();
//
//             let mut ser = Vec::new();
//             biome.serialize(&mut ser).unwrap();
//
//             let biome2 = Biomes::deserialize(ser.as_ref().as_ref()).unwrap();
//
//             assert_eq!(biome, biome2);
//         }
//     }
// }
//
// #[test]
// fn subchunks() {
//     let _lock = LOCK.lock().unwrap();
//     let database = unsafe {
//         Database::open("test/db").unwrap()
//     };
//     let iter = database.iter();
//     for kv in iter {
//         let key = kv.key();
//         if key[key.len() - 2] == 0x2f {
//             let subchunk = SubChunk::deserialize(&*kv.value()).unwrap();
//
//             let serialized = subchunk.serialize().unwrap();
//             let deserialized = SubChunk::deserialize(serialized.as_slice()).unwrap();
//
//             assert_eq!(subchunk, deserialized);
//         }
//     }
// }
//
// #[ignore]
// #[test]
// fn bench_subchunk() {
//     let _lock = LOCK.lock().unwrap();
//     let db = unsafe {
//         Database::open("test/db").unwrap()
//     };
//
//     let mut count = 0;
//     let mut failed = 0;
//     let mut sum = 0;
//
//     for _ in 0..50 {
//         let iter = db.iter();
//         for raw_ref in iter {
//             let key = raw_ref.key();
//
//             if key[key.len() - 2] == 0x2f {
//                 let start = std::time::Instant::now();
//                 let subchunk = SubChunk::deserialize(raw_ref.value().as_ref());
//                 let end = start.elapsed();
//
//                 if subchunk.is_ok() {
//                     count += 1;
//                     sum += end.as_micros();
//                 } else {
//                     failed += 1;
//                 }
//             }
//         }
//     }
//
//     let avg = sum as f64 / count as f64;
//     println!("average: {avg}μs");
//     println!("total chunks: {}", count as f64 / 1.0f64);
//     println!("failed: {} ({}%)", failed, failed as f64 / (count + failed) as f64);
//
//     // let mut buffer = OwnedBuffer::new();
//     // DatabaseKey {
//     //     x: -2,
//     //     z: -1,
//     //     data: KeyData::SubChunk {
//     //         index: 6
//     //     },
//     //     dimension: Dimension::Overworld
//     // }.serialize(&mut buffer);
//     //
//     // let data = db.get_raw_key(buffer).unwrap();
//     // let subchunk = SubChunk::deserialize(data).unwrap();
//     // let block = subchunk.get(Vector::from([0, 0, 0])).unwrap();
//     // dbg!(block);
// }
//
// #[test]
// fn palette_entry() {
//     let entry = PaletteEntry {
//         name: "minecraft:stone".to_owned(),
//         version: Some([1, 18, 100, 0]),
//         states: HashMap::from([("stone_type".to_owned(), nbt::Value::String("andesite".to_owned()))]),
//     };
//
//     let ser = nbt::to_le_bytes(&entry).unwrap();
//     let de: PaletteEntry = nbt::from_le_bytes(*ser.as_ref()).unwrap().0;
//     let _de_value: nbt::Value = nbt::from_le_bytes(*ser.as_ref()).unwrap().0;
//
//     assert_eq!(entry, de);
// }
