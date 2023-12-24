use crate::cache::CompilationCache;

use crate::{extension::Extension, ASSEMBLY_DIRECTORY, CACHE_DIRECTORY};
use anyhow::Context;
use std::ffi::OsStr;
use wasmtime::{Config, Engine, Linker, Store};
use wasmtime_wasi::WasiCtxBuilder;

/// The extension runtime.
///
/// This runtime takes care of compiling extensions and executing them.
pub struct ExtensionRuntime {
    /// WebAssembly compiler engine.
    engine: Engine,
    /// List of extensions that the runtime is tracking.
    extensions: Vec<Extension>,
}

impl ExtensionRuntime {
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
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let cache = CompilationCache::new(CACHE_DIRECTORY)?;
        let module_paths = std::fs::read_dir(ASSEMBLY_DIRECTORY)
            .context(format!(
                "Failed to read `{ASSEMBLY_DIRECTORY}` directory, please make sure the `extensions` directory exists."
            ))?
            .filter_map(|entry| {
                // Load only .wasm files
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() && entry.path().extension() == Some(OsStr::new("wasm")) {
                            return entry.file_name().into_string().ok();
                        }
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        let mut extensions = Vec::with_capacity(module_paths.len());
        for path in module_paths {
            let wasi = WasiCtxBuilder::new().inherit_stdio().build();
            let mut store = Store::new(&engine, wasi);

            let module = cache.load(&engine, &path).context(format!("Failed to compile module `{path}`"))?;
            linker
                .module(&mut store, &path, &module)
                .context(format!("Failed to link module `{path}`"))?;

            let instance = linker
                .instantiate(&mut store, &module)
                .context(format!("Failed to instantiate module `{path}`"))?;

            let path_clone = path.clone();
            let extension = Extension::new(path, instance, store).context(format!("Failed to initialize module `{path_clone}`"))?;

            extensions.push(extension);
        }

        if !extensions.is_empty() {
            tracing::info!("Initialised {} extension(s)", extensions.len());
        }

        // Run startup function for each extension.
        extensions.iter_mut().map(Extension::on_startup).for_each(|result| {
            if let Err(err) = result {
                tracing::error!("{err:?}");
            }
        });

        Ok(Self { engine, extensions })
    }
}

impl Drop for ExtensionRuntime {
    fn drop(&mut self) {
        self.extensions.drain(..).map(Extension::on_shutdown).for_each(|r| {
            if let Err(err) = r {
                tracing::error!("{err:?}");
            }
        });
    }
}
