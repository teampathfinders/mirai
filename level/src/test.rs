use bytes::BytesMut;
use common::{Deserialize, Serialize, Vector3b};

use crate::{
    database::ChunkDatabase, DatabaseKey, KeyData, Dimension, SubChunk, biome::Biome3d,
};

#[test]
fn database_test() {
    let db = ChunkDatabase::new("test/db").unwrap();

    for x in -25..25 {
        for z in -25..25 {
            let mut buffer = BytesMut::new();
            DatabaseKey {
                x,
                z,
                dimension: Dimension::Overworld,
                data: KeyData::Entity,
            }
            .serialize(&mut buffer);
            // println!("{:?}", buffer.as_ref());

            if let Ok(data) = db.get_raw_key(buffer.freeze()) {
                println!("{:?}", data.as_ref());
                break
            }
        }
    }

    // let biome = Biome3d::deserialize(data).unwrap();
    // println!("{biome:?}");

    // let sub_chunk = SubChunk::deserialize(data).unwrap();

    // println!("{sub_chunk:?}");
}
