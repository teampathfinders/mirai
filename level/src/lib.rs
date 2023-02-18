#[cfg(test)]
mod test;

mod database;
mod ffi;
mod sub_chunk;
mod world;

use database::ChunkDatabase;
pub use sub_chunk::*;
pub use world::*;

#[derive(Debug)]
pub struct ChunkManager {
    database: ChunkDatabase
}