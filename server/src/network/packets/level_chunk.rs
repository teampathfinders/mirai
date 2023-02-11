use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

// #[derive(Debug)]
// pub struct LevelChunk {
//     pub position: ChunkPosition,
//     pub subchunk_count: u32,
//     pub cache_enabled: bool,
//     pub chunk_data: BytesMut
// }

// impl GamePacket for LevelChunk {
//     const ID: u32 = 0x3a;
// }

// impl Encodable for LevelChunk {
//     fn encode(&self) -> VResult<BytesMut> {
//         let mut buffer = BytesMut::new();

//         buffer.put_var_i32(self.chunk_x);
//         buffer.put_var_i32(self.chunk_z);
//         buffer.put_var_u32(self.subchunk_count);
//         buffer.put_bool(self.cache_enabled);
//         buffer.put(self.chunk_data.as_ref());

//         Ok(buffer)
//     }
// }
