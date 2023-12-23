#![allow(rustdoc::private_intra_doc_links)]

/// Location of the WebAssembly modules.
const ASSEMBLY_DIRECTORY: &str = "extensions";
/// Location of the cache.
const CACHE_DIRECTORY: &str = "cache";

mod cache;
mod def;
mod extension;
mod runtime;
mod stdio;

pub use extension::*;
pub use runtime::*;
