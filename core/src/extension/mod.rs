mod cache;
mod vm;

pub use vm::*;
pub use cache::*;

/// Location of the WebAssembly modules.
const ASSEMBLY_DIRECTORY: &str = "ext";
/// Location of precompiled modules.
const CACHE_DIRECTORY: &str = "cache";