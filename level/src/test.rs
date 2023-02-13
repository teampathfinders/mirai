use common::{Decodable, Encodable};

use crate::{
    database::Database, DatabaseKey, DatabaseTag, Dimension, SubChunk,
};

#[test]
fn database_open() {
    let db = Database::new("test/db").unwrap();
    let key = DatabaseKey {
        x: 0,
        y: 0,
        z: 0,
        dimension: Dimension::Overworld,
        tag: DatabaseTag::SubChunk,
    }
    .encode()
    .unwrap();

    let data = db.get_raw_key(key).unwrap();
    let sub_chunk = SubChunk::decode(data).unwrap();

    println!("{sub_chunk:?}");
}
