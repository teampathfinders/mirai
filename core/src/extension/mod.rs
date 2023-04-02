//! The runtime and compiler used for server extensions.

mod cache;
mod ext;
mod runtime;

pub use cache::*;
pub use ext::*;
pub use runtime::*;

/// Location of the WebAssembly modules.
pub const ASSEMBLY_DIRECTORY: &str = "ext";
/// Location of the cache.
pub const CACHE_DIRECTORY: &str = "cache";
