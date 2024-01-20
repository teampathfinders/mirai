use util::Deserialize;
use util::MutableBuffer;
use util::Serialize;

use proto::bedrock::Header;

#[test]
fn biome_nbt() {
    let biomes: nbt::Value = nbt::from_var_bytes(include_bytes!("../include/biomes.nbt").as_ref()).unwrap().0;
    dbg!(biomes);
}

#[test]
fn header() {
    let header = Header {
        id: 129,
        sender_subclient: 3,
        target_subclient: 2,
    };

    let mut buffer = MutableBuffer::new();
    header.serialize_into(&mut buffer).unwrap();

    assert_eq!(Header::deserialize(buffer.as_ref()).unwrap(), header);
}
