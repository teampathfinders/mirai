

use dashmap::DashMap;
use util::bytes::{ArcBuffer, MutableBuffer, SharedBuffer};

use crate::network::raknet::Frame;
use crate::network::raknet::Reliability;

/// Keeps track of packet fragments, merging them when all fragments have been received.
#[derive(Debug, Default)]
pub struct CompoundCollector {
    compounds: DashMap<u16, Vec<MutableBuffer>>,
}

impl CompoundCollector {
    /// Creates a new collector.
    pub fn new() -> Self {
        Self { compounds: DashMap::new() }
    }

    /// Inserts a fragment into the collector.
    ///
    /// If this fragment makes the compound complete, all fragments will be merged
    /// and the completed packet will be returned.
    pub fn insert(&self, mut frame: Frame) -> Option<Frame> {
        let is_completed = {
            let mut entry =
                self.compounds.entry(frame.compound_id).or_insert_with(|| {
                    let mut vec =
                        Vec::with_capacity(frame.compound_size as usize);

                    vec.resize_with(frame.compound_size as usize, || MutableBuffer::new());
                    vec
                });

            let mut fragments = entry.value_mut();

            // Verify that the fragment index is valid
            if frame.compound_index >= frame.compound_size {
                return None;
            }

            fragments[frame.compound_index as usize] = frame.body;
            !fragments.iter().any(<[u8]>::is_empty)
        };

        if is_completed {
            let mut kv = self
                .compounds
                .remove(&frame.compound_id)
                .expect("Compound ID was not found in collector");

            let fragments = &mut kv.1;

            // Merge all fragments
            frame.body = SharedBuffer::copy_from_slice(fragments.concat().as_slice());

            // Set compound tag to false to make sure the completed packet isn't added into the
            // collector again.
            frame.is_compound = false;
            // Set reliability to unreliable to prevent duplicated acknowledgements
            // frame.reliability = Reliability::Unreliable;

            return Some(frame);
        }

        None
    }
}
