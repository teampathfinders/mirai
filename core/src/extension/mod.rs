//! This module contains code used to run extensions/plugins.

mod cache;
mod ext;
mod runtime;

pub use cache::*;
pub use ext::*;
pub use runtime::*;

/// Location of the WebAssembly modules.
pub const ASSEMBLY_DIRECTORY: &str = "ext";
/// Location of precompiled modules.
pub const CACHE_DIRECTORY: &str = "cache";
