use std::{
    collections::HashMap,
    num::NonZeroU16,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use dashmap::DashMap;
use nohash_hasher::BuildNoHashHasher;
use util::{RVec, Vector};

use crate::level::io::io::RegionIndex;

pub struct BlobRef {
    counter: AtomicU16,
    blob: Arc<RVec>,
}

pub struct BlobCache {
    /// Blob data stored by hash.
    /// As clients will request by hash rather than coordinates,
    /// this means there will only be one lookup.
    blobs: DashMap<u64, BlobRef, BuildNoHashHasher<u64>>,
    /// Maps coordinates to blob hashes.
    pos_to_hash: DashMap<u64, u64, BuildNoHashHasher<u64>>,
    hash_to_pos: DashMap<u64, u64, BuildNoHashHasher<u64>>,
}

impl BlobCache {
    pub fn new() -> BlobCache {
        BlobCache {
            blobs: DashMap::with_hasher(BuildNoHashHasher::default()),
            pos_to_hash: DashMap::with_hasher(BuildNoHashHasher::default()),
            hash_to_pos: DashMap::with_hasher(BuildNoHashHasher::default()),
        }
    }

    /// Inserts a blob into the cache.
    /// If a blob already existed for this chunk, it will be removed and returned.
    pub fn cache(&self, position: Vector<i32, 3>, blob: RVec, hash: u64) -> anyhow::Result<Option<Arc<RVec>>> {
        let index = RegionIndex::try_from(position)?;

        self.blobs.insert(
            hash,
            BlobRef {
                counter: AtomicU16::new(1),
                blob: Arc::new(blob),
            },
        );

        // A blob already exists for this chunk, remove it and return it.
        if let Some(prev_hash) = self.pos_to_hash.insert(index.into(), hash) {
            let prev_blob = self.blobs.remove(&prev_hash);
            assert!(prev_blob.is_some(), "Missing blob for hash in index map");

            return Ok(prev_blob.map(|(_, x)| x.blob));
        }

        Ok(None)
    }

    /// Removes one from the reference counter of the given blob, returning the *previous*
    /// reference count if the blob was found.
    ///
    /// If the reference count has reached zero, the blob is removed.
    pub fn unref_by_hash(&self, hash: u64) -> Option<NonZeroU16> {
        let count = self.blobs.get(&hash).map(|kv| {
            let prev = kv.value().counter.fetch_sub(1, Ordering::Relaxed);
            NonZeroU16::new(prev).expect("Previous reference count was 0, this is impossible")
        })?;

        // Blob should be removed since count reached zero.
        if count.get() == 1 {
            let _ = self.blobs.remove(&hash);
            tracing::debug!("Removed blob {hash}");

            let pos = self.hash_to_pos.remove(&hash).expect("Missing hash to position entry").1;
            let _ = self.pos_to_hash.remove(&pos);
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
    pub fn get_by_pos(&self, pos: Vector<i32, 3>) -> anyhow::Result<Option<Arc<RVec>>> {
        let Some(hash) = self.get_hash(pos)? else { return Ok(None) };
        self.get_by_hash(hash)
    }

    /// Retrieves the hash of a chunk without returning the blob.
    #[inline]
    pub fn get_hash(&self, pos: Vector<i32, 3>) -> anyhow::Result<Option<u64>> {
        let index = RegionIndex::try_from(pos)?;
        Ok(self.pos_to_hash.get(&index.into()).map(|kv| *kv.value()))
    }

    /// Loads a blob using its hash, increasing its reference count by one.
    pub fn get_by_hash(&self, hash: u64) -> anyhow::Result<Option<Arc<RVec>>> {
        let Some(blob_ref) = self.blobs.get(&hash) else {
            anyhow::bail!("Missing blob for hash in index map");
        };

        blob_ref.counter.fetch_add(1, Ordering::Relaxed);
        Ok(Some(blob_ref.blob.clone()))
    }
}
