use crate::extension::{CompilationCache, Extension, ASSEMBLY_DIRECTORY, CACHE_DIRECTORY};
use anyhow::Context;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use wasmtime::{Config, Engine, Instance, Module, Store};

pub struct Runtime {
    engine: Engine,
    store: Store<()>,
    extensions: Vec<Extension>,
}

impl Runtime {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        config.parallel_compilation(true);

        let engine = Engine::new(&config).context("Failed to create engine")?;

        let cache = CompilationCache::new(CACHE_DIRECTORY)?;
        let module_paths = std::fs::read_dir(ASSEMBLY_DIRECTORY)?
            .filter_map(|entry| {
                // Load only .wasm files
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() && entry.path().extension() == Some(&OsStr::new("wasm")) {
                            return entry.file_name().into_string().ok();
                        }
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        let mut store = Store::new(&engine, ());
        let mut extensions = Vec::with_capacity(module_paths.len());
        for path in &module_paths {
            let module = cache.load(&engine, path).context(format!("Failed to compile extension {path}"))?;

            let instance = Instance::new(&mut store, &module, &[])?;
            let extension = Extension::new(instance, &mut store)?;

            extensions.push(extension);
        }

        tracing::info!("Extension runtime initialised");
        Ok(Self { engine, store, extensions })
    }
}
