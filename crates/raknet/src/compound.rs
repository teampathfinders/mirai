use std::io::Write;

use dashmap::DashMap;
use util::RVec;

use crate::Frame;

/// Keeps track of packet fragments, merging them when all fragments have been received.
#[derive(Default, Debug)]
pub struct Compounds {
    compounds: DashMap<u16, Vec<Option<Frame>>>,
}

impl Compounds {
    /// Creates a new collector.
    pub fn new() -> Compounds {
        Compounds { compounds: DashMap::new() }
    }

    /// Inserts a fragment into the collector.
    ///
    /// If this fragment makes the compound complete, all fragments will be merged
    /// and the completed packet will be returned.
    #[allow(clippy::unwrap_used)] // Checks are performed before unwrapping.
    #[allow(clippy::unwrap_in_result)]
    #[allow(clippy::significant_drop_tightening)] // False positive.
    #[allow(clippy::missing_panics_doc)] // Function should not panic.
    pub fn insert(&self, frame: Frame) -> anyhow::Result<Option<Frame>> {
        // Save compound_id, because the frame will be moved.
        let compound_id = frame.compound_id;
        let is_completed = {
            if frame.compound_index >= frame.compound_size {
                return Ok(None)
            }

            // Save compound_index, because frame is moved by the Some constructor.
            let compound_index = frame.compound_index as usize;

            let mut entry = self.compounds.entry(frame.compound_id).or_insert_with(|| {
                let mut vec = Vec::with_capacity(frame.compound_size as usize);

                // resize_with instead of resize, because Frame does not implement Clone
                vec.resize_with(frame.compound_size as usize, || None);
                vec
            });

            let fragments = entry.value_mut();
            fragments[compound_index] = Some(frame);

            // Verify that the fragment index is valid
            !fragments.iter().any(Option::is_none)
        };

        if is_completed {
            let mut kv = self
                .compounds
                .remove(&compound_id)
                .unwrap();

            let fragments = &mut kv.1;

            // Merge all fragments
            let mut merged = RVec::alloc_with_capacity(
                fragments
                    .iter()
                    .fold(0, |acc, f| acc + f.as_ref().unwrap().body.len())
            );

            let mut failed = None;
            fragments
                .iter()
                .for_each(|b| if let Some(b) = b {
                    if let Err(e) = merged.write_all(b.body.as_slice()) {
                        failed = Some(e);
                    }
                });

            if let Some(e) = failed {
                return Err(e.into());
            }

            let mut frame = fragments[0].take().unwrap();
            frame.body = merged;

            // Set compound tag to false to make sure the completed packet isn't added into the
            // collector again.
            frame.is_compound = false;
            // Set reliability to unreliable to prevent duplicated acknowledgements
            // frame.reliability = Reliability::Unreliable;

            return Ok(Some(frame));
        }

        Ok(None)
    }
}
