/// Location of the WebAssembly modules.
const ASSEMBLY_DIRECTORY: &str = "plugins";
/// Location of the cache.
const CACHE_DIRECTORY: &str = "cache";

mod cache;
mod def;
mod plugin;
mod runtime;
mod stdio;

pub use plugin::*;
pub use runtime::*;
