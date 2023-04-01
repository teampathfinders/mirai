//! This module contains code used to run extensions/plugins.

mod cache;
mod vm;

pub use vm::*;
pub use cache::*;

/// Location of the WebAssembly modules.
pub const ASSEMBLY_DIRECTORY: &str = "ext";
/// Location of precompiled modules.
pub const CACHE_DIRECTORY: &str = "cache";