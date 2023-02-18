use common::{Deserialize, Serialize};

use crate::{
    database::ChunkDatabase, DatabaseKey, DatabaseTag, Dimension, SubChunk,
};

#[test]
fn database_open() {
    let db = ChunkDatabase::new("test/db").unwrap();
    let key = DatabaseKey {
        x: 0,
        y: 0,
        z: 0,
        dimension: Dimension::Overworld,
        tag: DatabaseTag::SubChunk,
    }
    .serialize()
    .unwrap();

    let data = db.get_raw_key(key).unwrap();
    let sub_chunk = SubChunk::deserialize(data).unwrap();

    println!("{sub_chunk:?}");
}
