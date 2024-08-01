use std::ops::Range;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI32, AtomicU16, Ordering},
        Arc,
    },
};

use futures::{future, StreamExt};
use level::SubChunk;
use nohash_hasher::BuildNoHashHasher;
use proto::bedrock::{CacheBlobStatus, NetworkChunkPublisherUpdate, SubChunkRequest};
use proto::{
    bedrock::{HeightmapType, LevelChunk, SubChunkEntry, SubChunkRequestMode, SubChunkResponse, SubChunkResult},
    types::Dimension,
};
use util::{Deserialize, RVec, Vector};

use crate::level::net::column::ChunkColumn;
use crate::net::BedrockClient;

pub type ChunkOffset = Vector<i8, 3>;

const USE_SUBCHUNK_REQUESTS: bool = true;

impl BedrockClient {
    /// Loads chunks around a center point.
    pub fn load_chunks(&self, center: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<()> {
        const VERTICAL_RANGE: Range<i16> = -4..20;

        let mut column = ChunkColumn::empty(center.clone());
        for y in VERTICAL_RANGE {
            let opt = self.level.subchunk((center.x, y as i32, center.y), dimension)?;
            let adj_y = (y - VERTICAL_RANGE.start) as u16;
            column.subchunks.push((adj_y, opt));
        }

        column.generate_heightmap();

        if self.supports_cache() {
            self.send_blob_hashes(center, &column, dimension)?;
        } else {
        }

        Ok(())
    }

    /// Sends blob hashes of the chunks that the client requested.
    fn send_blob_hashes(&self, coordinates: Vector<i32, 2>, column: &ChunkColumn, dimension: Dimension) -> anyhow::Result<()> {
        use xxhash_rust::xxh64::xxh64;

        if USE_SUBCHUNK_REQUESTS {
            let biomes = column.serialize_biomes()?;
            // Blob cache uses 64-bit xxHash with seed 0.
            let hash = xxh64(&biomes, 0);

            let pk = LevelChunk {
                dimension,
                coordinates,
                request_mode: SubChunkRequestMode::KnownAir { highest_nonair: column.highest_nonair() },
                // blob_hashes: Some(vec![hash]),
                blob_hashes: None,
                raw_payload: RVec::alloc_from_slice(&[0]),
            };
            tracing::debug!("{pk:?}");
            self.send(pk)?;
            tracing::debug!("Sent LevelChunk");

            self.send(NetworkChunkPublisherUpdate { position: (0, 0, 0).into(), radius: 12 })?;
        } else {
            todo!()
        }

        Ok(())
    }

    pub fn handle_cache_blob_status(&self, packet: RVec) -> anyhow::Result<()> {
        let status = CacheBlobStatus::deserialize(packet.as_ref())?;
        dbg!(status);

        Ok(())
    }

    pub fn handle_subchunk_request(&self, packet: RVec) -> anyhow::Result<()> {
        let request = SubChunkRequest::deserialize(packet.as_ref())?;
        tracing::debug!("request: {request:?}");

        let mut entries = Vec::with_capacity(request.offsets.len());
        for offset in request.offsets {
            let entry = SubChunkEntry {
                result: SubChunkResult::Success,
                offset,
                // Get subchunk data
            };

            entries.push(entry);
        }

        let response = SubChunkResponse {
            cache_enabled: self.supports_cache(),
            dimension: request.dimension,
            position: request.position,
            entries,
        };

        self.send(response)
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
