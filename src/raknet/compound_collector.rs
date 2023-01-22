use bytes::BytesMut;
use dashmap::DashMap;

use crate::raknet::Frame;

/// Keeps track of packet fragments, merging them when all fragments have been received.
#[derive(Debug)]
pub struct CompoundCollector {
    compounds: DashMap<u16, Vec<BytesMut>>,
}

impl CompoundCollector {
    /// Creates a new collector.
    pub fn new() -> Self {
        Self {
            compounds: DashMap::new(),
        }
    }

    /// Inserts a fragment into the collector.
    ///
    /// If this fragment makes the compound complete, all fragments will be merged
    /// and the completed packet will be returned.
    pub fn insert(&self, fragment: Frame) -> Option<BytesMut> {
        let mut fragment_list = self
            .compounds
            .entry(fragment.compound_id)
            .or_insert_with(|| {
                let mut vec = Vec::with_capacity(fragment.compound_size as usize);
                vec.resize(fragment.compound_size as usize, BytesMut::new());

                vec
            });

        fragment_list.insert(fragment.compound_index as usize, fragment.body);

        let is_complete = !fragment_list.iter().any(BytesMut::is_empty);
        if is_complete {
            let final_size = fragment_list.iter().fold(0, |acc, f| acc + f.len());
            let mut final_buffer = BytesMut::with_capacity(final_size);

            for buffer in fragment_list.value() {
                final_buffer.extend_from_slice(buffer);
            }

            self.compounds.remove(&fragment.compound_id);
            Some(final_buffer)
        } else {
            None
        }
    }
}
