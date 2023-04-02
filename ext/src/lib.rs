/// Location of the WebAssembly modules.
const ASSEMBLY_DIRECTORY: &str = "ext";
/// Location of the cache.
const CACHE_DIRECTORY: &str = "cache";

mod cache;
mod extension;
mod runtime;
mod stdio;

pub use runtime::*;
pub use extension::*;