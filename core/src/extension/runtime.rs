use crate::extension::{CompilationCache, Extension, ASSEMBLY_DIRECTORY, CACHE_DIRECTORY};
use anyhow::Context;
use std::ffi::OsStr;
use wasmtime::{Config, Engine, Instance, Store};

/// The extension runtime.
/// 
/// This runtime takes care of compiling extensions and executing them.
pub struct Runtime {
    /// WebAssembly compiler engine.
    engine: Engine,
    /// Contains all module data.
    store: Store<()>,
    /// List of extensions that the runtime is tracking.
    extensions: Vec<Extension>,
}

impl Runtime {
    /// Creates a new extension runtime.
    /// 
    /// This function checks the [`ASSEMBLY_DIRECTORY`] path for extensions to compile.
    /// The modules can either be compiled or loaded from cache, see [`CompilationCache`] and [`CompilationCache::load`] for more
    /// information. 
    /// 
    /// After compiling a module, it will be registered as an extension. See [`Extension`] for more information.
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
