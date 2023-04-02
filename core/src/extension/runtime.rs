use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use anyhow::Context;
use wasmtime::{Config, Engine, Instance, Module, Store};
use crate::extension::{ASSEMBLY_DIRECTORY, CACHE_DIRECTORY, CompilationCache, Extension};

pub struct Runtime {
    engine: Engine,
    store: Store<()>,
    modules: Vec<Extension>
}

impl Runtime {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        config.parallel_compilation(true);

        let engine = Engine::new(&config)
            .context("Failed to create engine")?;

        let cache = CompilationCache::new(CACHE_DIRECTORY)?;
        let module_paths = std::fs::read_dir(ASSEMBLY_DIRECTORY)?
            .filter_map(|entry| {
                // Load only .wasm files
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() && entry.path().extension() == Some(&OsStr::new("wasm")) {
                            return entry.file_name().into_string().ok()
                        }
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        let mut modules = Vec::with_capacity(module_paths.len());
        for path in &module_paths {
            let module = cache.load(&engine, path)
                .context(format!("Failed to compile extension {path}"))?;

            modules.push(module);
        }

        let mut store = Store::new(&engine, ());

        todo!()
    }
}