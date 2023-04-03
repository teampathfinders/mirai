use crate::cache::CompilationCache;

use crate::{ASSEMBLY_DIRECTORY, CACHE_DIRECTORY, plugin::Plugin};
use anyhow::Context;
use wasmtime_wasi::{stdio::stdout, WasiCtxBuilder, WasiCtx};
use std::ffi::OsStr;
use wasmtime::{Config, Engine, Instance, Store, Linker};

/// The extension runtime.
/// 
/// This runtime takes care of compiling plugins and executing them.
pub struct PluginRuntime {
    /// WebAssembly compiler engine.
    engine: Engine,
    /// List of plugins that the runtime is tracking.
    plugins: Vec<Plugin>,
}

impl PluginRuntime {
    /// Creates a new plugin runtime.
    /// 
    /// This function checks the [`ASSEMBLY_DIRECTORY`] path for plugins to compile.
    /// The modules can either be compiled or loaded from cache, see [`CompilationCache`] and [`CompilationCache::load`] for more
    /// information. 
    /// 
    /// After compiling a module, it will be registered as a plugin. See [`Extension`] for more information.
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        config.parallel_compilation(true);

        let engine = Engine::new(&config).context("Failed to create engine")?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

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

        let mut plugins = Vec::with_capacity(module_paths.len());
        for path in module_paths {
            let wasi = WasiCtxBuilder::new()
                .inherit_stdio()
                .build();
            let mut store = Store::new(&engine, wasi);

            let module = cache.load(&engine, &path).context(format!("Failed to compile module `{path}`"))?;
            linker.module(&mut store, &path, &module)
                .context(format!("Failed to link module `{path}`"))?;

            let instance = linker.instantiate(&mut store, &module)
                .context(format!("Failed to instantiate module `{path}`"))?;

            let path_clone = path.clone();
            let plugin = Plugin::new(path, instance, store)
                .context(format!("Failed to initialize module `{path_clone}`"))?;

            plugins.push(plugin);
        }

        tracing::info!("Extension runtime initialised");
        Ok(Self { engine, plugins })
    }

    /// Calls the shutdown callback for every plugin and drops all plugins.
    pub fn shutdown(self) {
        self.plugins
            .into_iter()
            .map(Plugin::on_shutdown)
            .for_each(|r| if let Err(err) = r { tracing::error!("{err}"); });
    }
}
