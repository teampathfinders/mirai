use std::{
    num::NonZeroU16,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use dashmap::DashMap;
use nohash_hasher::BuildNoHashHasher;
use util::{RVec, Vector};
use xxhash_rust::xxh64::xxh64;

use crate::level::io::io::RegionIndex;

use super::net::heightmap::Heightmap;

#[derive(Debug, Clone)]
pub struct CacheableSubChunk {
    pub hash: u64,
    pub heightmap: Heightmap,
    pub payload: Arc<RVec>,
}

#[derive(Debug)]
pub struct ChunkRef {
    counter: AtomicU16,
    data: Arc<CacheableSubChunk>,
}

#[derive(Debug)]
pub struct BlobCache {
    /// Chunk data stored by hash.
    /// As clients will request by hash rather than coordinates,
    /// this means there will only be one lookup.
    chunks: DashMap<u64, ChunkRef, BuildNoHashHasher<u64>>,
    /// Maps coordinates to blob hashes.
    pos_to_hash: DashMap<u64, u64, BuildNoHashHasher<u64>>,
}

impl BlobCache {
    pub fn new() -> BlobCache {
        BlobCache {
            chunks: DashMap::with_hasher(BuildNoHashHasher::default()),
            pos_to_hash: DashMap::with_hasher(BuildNoHashHasher::default()),
        }
    }

    /// Inserts a blob into the cache.
    /// If a blob already existed for this chunk, it will be removed and returned.
    pub fn cache(&self, position: Vector<i32, 3>, heightmap: Heightmap, payload: RVec) -> anyhow::Result<Option<Arc<CacheableSubChunk>>> {
        let index = RegionIndex::try_from(position)?;
        let hash = xxh64(&payload, 0);

        let chunk = CacheableSubChunk {
            hash,
            heightmap,
            payload: Arc::new(payload),
        };

        self.chunks.insert(
            hash,
            ChunkRef {
                counter: AtomicU16::new(1),
                data: Arc::new(chunk),
            },
        );

        // A blob already exists for this chunk, remove it and return it.
        if let Some(prev_hash) = self.pos_to_hash.insert(index.into(), hash) {
            let prev_blob = self.chunks.remove(&prev_hash);
            assert!(prev_blob.is_some(), "Missing chunk for hash in index map");

            return Ok(prev_blob.map(|(_, x)| x.data));
        }

        Ok(None)
    }

    /// Removes one from the reference counter of the given blob, returning the *previous*
    /// reference count if the blob was found.
    ///
    /// If the reference count has reached zero, the blob is removed.
    pub fn unref_by_hash(&self, hash: u64) -> Option<NonZeroU16> {
        let count = self.chunks.get(&hash).map(|kv| {
            let prev = kv.value().counter.fetch_sub(1, Ordering::Relaxed);
            NonZeroU16::new(prev).expect("Previous reference count was 0, this is impossible")
        })?;

        // Blob should be removed since count reached zero.
        if count.get() == 1 {
            let _ = self.chunks.remove(&hash);
            tracing::debug!("Removed chunk {hash}");

            // There may be multiple chunks referring to the same hash.
            // We therefore iterate over the entire map to remove all of them.
            self.pos_to_hash.retain(|k, v| {
                tracing::debug!("Removed reference {k:?} => {v}");
                *v != hash
            });
        }

        Some(count)
    }

    /// Removes one from the reference count of the given blob, returning the *previous*
    /// reference count if the blob was found.
    ///
    /// If the reference count has reached zero, the blob is removed.
    pub fn unref_by_pos(&self, pos: Vector<i32, 3>) -> anyhow::Result<Option<NonZeroU16>> {
        let Some(hash) = self.get_hash(pos)? else { return Ok(None) };
        Ok(self.unref_by_hash(hash))
    }

    /// Attempts to load a blob by coordinate.
    /// If there does not exist a blob for the requested location,
    /// the chunk should be loaded from disk and a new blob should be created using
    /// the [`cache`](Self::cache) method.
    ///
    /// If there does exist a blob, it will be returned and the reference count increased by one.
    ///
    /// Internally this simply retrieves the chunk's hash using [`get_hash`](Self::get_hash)
    /// and calls [`get_by_hash`](Self::get_by_hash) to load the blob.
    pub fn get_by_pos(&self, pos: Vector<i32, 3>) -> anyhow::Result<Option<Arc<CacheableSubChunk>>> {
        let Some(hash) = self.get_hash(pos)? else { return Ok(None) };
        self.get_by_hash(hash, true)
    }

    /// Retrieves the hash of a chunk without returning the blob.
    #[inline]
    pub fn get_hash(&self, pos: Vector<i32, 3>) -> anyhow::Result<Option<u64>> {
        let index = RegionIndex::try_from(pos)?;
        Ok(self.pos_to_hash.get(&index.into()).map(|kv| *kv.value()))
    }

    /// Loads a blob using its hash.
    /// The reference counter of the chunk can optionally be incremented.
    pub fn get_by_hash(&self, hash: u64, incr_ref: bool) -> anyhow::Result<Option<Arc<CacheableSubChunk>>> {
        let Some(chunk_ref) = self.chunks.get(&hash) else {
            anyhow::bail!("Missing chunk for hash in index map");
        };

        if incr_ref {
            chunk_ref.counter.fetch_add(1, Ordering::Relaxed);
        }

        Ok(Some(chunk_ref.data.clone()))
    }
}
