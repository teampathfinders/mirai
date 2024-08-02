use std::ops::Range;
use std::sync::Arc;

use proto::bedrock::{CacheBlobStatus, NetworkChunkPublisherUpdate, SubChunkRequest};
use proto::{
    bedrock::{LevelChunk, SubChunkEntry, SubChunkRequestMode, SubChunkResponse, SubChunkResult},
    types::Dimension,
};
use util::{Deserialize, RVec, Vector};
use xxhash_rust::xxh64::xxh64;

use crate::level::blobs::CacheableSubChunk;
use crate::level::net::column::ChunkColumn;
use crate::level::net::heightmap::Heightmap;
use crate::level::net::ser::NetworkChunkExt;
use crate::net::BedrockClient;

pub type ChunkOffset = Vector<i8, 3>;

const USE_SUBCHUNK_REQUESTS: bool = true;
pub const CHUNK_VERTICAL_RANGE: Range<i8> = -4..20;

impl BedrockClient {
    fn load_column(&self, center: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<ChunkColumn> {
        let mut column = ChunkColumn::empty(center.clone(), CHUNK_VERTICAL_RANGE);
        for y in CHUNK_VERTICAL_RANGE {
            let opt = self.level.subchunk((center.x, y as i32, center.y), dimension)?;
            // let adj_y = (y - CHUNK_VERTICAL_RANGE.start) as u16
            column.subchunks.push((y as i16, opt));
        }

        column.generate_heightmap();

        Ok(column)
    }

    /// Loads chunks around a center point.
    pub fn initiate_chunk_load(&self, center: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<()> {
        let column = self.load_column(center.clone(), dimension)?;

        if self.supports_cache() {
            self.send_chunk_skeleton(center, &column, dimension)?;
        } else {
            todo!()
        }

        Ok(())
    }

    /// Initiates the subchunk request process in a chunk column.
    fn send_chunk_skeleton(&self, pos: Vector<i32, 2>, column: &ChunkColumn, dimension: Dimension) -> anyhow::Result<()> {
        self.send(LevelChunk {
            dimension,
            coordinates: pos.clone(),
            request_mode: SubChunkRequestMode::KnownAir { highest_nonair: column.highest_nonair() },
            // blob_hashes: Some(vec![hash]),
            blob_hashes: None,
            raw_payload: RVec::alloc_from_slice(&[0]),
        })?;
        self.send(NetworkChunkPublisherUpdate {
            position: (pos.x, 0, pos.y).into(),
            radius: 12,
        })
        // TODO: Remember render distance.
    }

    fn load_subchunk(&self, pos: Vector<i32, 3>, dimension: Dimension) -> anyhow::Result<Arc<CacheableSubChunk>> {
        if let Some(cached) = self.level.blobs().get_by_pos(pos.clone())? {
            Ok(cached)
        } else {
            // Subchunk is unknown, load and cache entire chunk column for heightmap generation.
            let column = self.load_column((pos.x, pos.z).into(), dimension)?;
            for (i, entry) in column.subchunks.iter().filter_map(|(i, x)| x.as_ref().map(|y| (i, y))) {
                let sub_pos = (pos.x, *i as i32, pos.z).into();
                dbg!(&sub_pos);
                let payload = entry.serialize_network(&self.instance().block_states)?;
                let heightmap = Heightmap::new(*i as i8 + CHUNK_VERTICAL_RANGE.start, &column);

                let cacheable = CacheableSubChunk { heightmap, payload };
                let _ = self.level.blobs().cache(sub_pos, cacheable);
            }

            // Request subchunk now that it has been properly cached.
            // It is loaded via the blob cache to make sure it is wrapped in an `Arc`.
            Ok(self
                .level
                .blobs()
                .get_by_pos(pos)?
                .ok_or_else(|| anyhow::anyhow!("Subchunk cached just now somehow unavailable"))?)
        }
    }

    pub fn handle_cache_blob_status(&self, packet: RVec) -> anyhow::Result<()> {
        let status = CacheBlobStatus::deserialize(packet.as_ref())?;
        dbg!(status);

        Ok(())
    }

    pub fn handle_subchunk_request(&self, packet: RVec) -> anyhow::Result<()> {
        let request = SubChunkRequest::deserialize(packet.as_ref())?;
        tracing::debug!("request: {request:?}");

        for offset in &request.offsets {
            let abs = (
                request.position.x + offset.x as i32,
                0 + offset.y as i32,
                request.position.z + offset.z as i32,
            )
                .into();
            dbg!(&abs);

            let subchunk = self.load_subchunk(abs, request.dimension)?;
        }

        todo!()

        // let center = (request.position.x, request.position.z).into();
        // let column = self.load_column(center, request.dimension)?;

        // let mut entries = Vec::with_capacity(request.offsets.len());
        // for offset in request.offsets {
        //     let index = (offset.y as i16 - column.range.start) as u16;
        //     let (_, Some(subchunk)) = &column.subchunks[index as usize] else {
        //         continue;
        //     };

        //     // Generate the heightmap for this subchunk.
        //     let heightmap = Heightmap::new(offset.y, &column);
        //     let mut writer = RVec::alloc();
        //     subchunk.serialize_network_in(&self.instance().block_states, &mut writer)?;

        //     let hash = xxh64(&writer, 0);
        //     let entry = SubChunkEntry {
        //         result: SubChunkResult::Success,
        //         offset,
        //         heightmap_type: heightmap.map_type,
        //         heightmap: heightmap.data, // Get subchunk data
        //         blob_hash: hash,
        //         payload: RVec::alloc_from_slice(&[0]),
        //     };

        //     entries.push(entry);
        // }

        // let response = SubChunkResponse {
        //     cache_enabled: self.supports_cache(),
        //     dimension: request.dimension,
        //     position: request.position,
        //     entries,
        // };

        // self.send(response)
    }
}

// use proto::bedrock::{GameRule, ParsedCommand};
// use util::TryExpect;

// pub const DEFAULT_EFFECT_DURATION: i32 = 30;

// impl Level {
//     pub fn on_gamerule_command(&self, _caller: u64, command: ParsedCommand) -> anyhow::Result<String> {
//         debug_assert_eq!(command.name, "gamerule");

//         // Parsing should already verify that these parameters are provided.
//         debug_assert!(command.parameters.contains_key("rule"));

//         let rule_name = command.parameters.get("rule")
//             // Rule parameter should exist, but this is here just to be sure.
//             .unwrap()
//             .as_string()
//             .try_expect("Expected `rule` of type String")?;

//         // Command has value parameter, store the game rule value.
//         if let Some(value) = command.parameters.get("value") {
//             let new_value = GameRule::from_parsed(rule_name, value)?;
//             let old_value = self.set_game_rule(new_value)?;

//             if let Some(old_value) = old_value {
//                 Ok(format!("Set game rule '{rule_name}' to {new_value} (was {old_value})."))
//             } else {
//                 Ok(format!("Set game rule '{rule_name}' to {new_value} (was not set)."))
//             }
//         } else {
//             // Command has no value parameter, load the game rule value.
//             if let Some(value) = self.get_game_rule(rule_name) {
//                 Ok(format!("Game rule '{rule_name}' is set to {value}"))
//             } else {
//                 Ok(format!("Game rule '{rule_name}' is not set"))
//             }
//         }
//     }

//     pub fn on_effect_command(&self, _caller: u64, command: ParsedCommand) -> anyhow::Result<String> {
//         debug_assert_eq!(command.name, "effect");

//         // Parsing should already verify that these parameters are provided.
//         debug_assert!(command.parameters.contains_key("effect"));
//         debug_assert!(command.parameters.contains_key("target"));

//         let effect_name = command.parameters.get("effect")
//             .unwrap()
//             .as_string()
//             .try_expect("Expected `effect` of type String")?;

//         if effect_name == "clear" {
//             // TODO: Specify names of entities.
//             Ok("Took all effects from entities".to_owned())
//         } else {
//             // If there's no duration, apply a default 30 seconds
//             let duration = if let Some(duration) = command.parameters.get("duration") {
//                 duration.as_int().try_expect("Expected `duration` of type Int")?
//             } else {
//                 DEFAULT_EFFECT_DURATION
//             };

//             let amplifier = if let Some(amplifier) = command.parameters.get("amplifier") {
//                 amplifier.as_int().try_expect("Expected `amplifier` of type Int")?
//             } else {
//                 1
//             };

//             let _hide_particles = if let Some(hide_particles) = command.parameters.get("hideParticles") {
//                 let h = hide_particles.as_string().try_expect("Expected `hideParticles` of type String")?;
//                 h == "true"
//             } else {
//                 false
//             };

//             Ok(format!("Applied {} * {} for {} seconds", effect_name, amplifier, duration))
//         }
//     }
// }
