/// Location of the WebAssembly modules.
const ASSEMBLY_DIRECTORY: &str = "plugins";
/// Location of the cache.
const CACHE_DIRECTORY: &str = "cache";

mod cache;
mod plugin;
mod runtime;
mod stdio;

pub use runtime::*;
pub use plugin::*;