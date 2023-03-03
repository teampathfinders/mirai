use bytes::BytesMut;
use common::{Deserialize, Serialize, Vector3b};

use crate::{
    database::ChunkDatabase, DatabaseKey, KeyData, Dimension, SubChunk,
};

#[test]
fn database_test() {
    let db = ChunkDatabase::new("test/db").unwrap();

    let mut buffer = BytesMut::new();
    DatabaseKey {
        x: 2,
        y: 5,
        z: 0,
        dimension: Dimension::Overworld,
        data: KeyData::Biome3d,
    }
    .serialize(&mut buffer);

    println!("{:?}", buffer.as_ref());

    // let data = db.get_raw_key(buffer.freeze()).unwrap();
    // println!("{:?}", data.as_ref());

    // let sub_chunk = SubChunk::deserialize(data).unwrap();

    // println!("{sub_chunk:?}");
}
