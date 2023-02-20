use bytes::BytesMut;
use common::{Deserialize, Serialize, Vector3b};

use crate::{
    database::ChunkDatabase, DatabaseKey, DatabaseTag, Dimension, SubChunk,
};

#[test]
fn database_open() {
    let db = ChunkDatabase::new("test/db").unwrap();

    let mut buffer = BytesMut::new();
    let key = DatabaseKey {
        x: 0,
        y: 3,
        z: 2,
        dimension: Dimension::Overworld,
        tag: DatabaseTag::SubChunk,
    }
    .serialize(&mut buffer);

    let data = db.get_raw_key(buffer.freeze()).unwrap();
    let sub_chunk = SubChunk::deserialize(data).unwrap();

    // let block = sub_chunk.get(Vector3b::from([]))

    println!("{:?}", sub_chunk.get(Vector3b::from([13, 5, 6])));
}
