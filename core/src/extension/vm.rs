use std::path::{Path, PathBuf};
use wasmer::{Cranelift, CraneliftOptLevel, Engine, imports, Instance, Store};
use crate::extension::cache::ModuleCache;
use util::Result;
use crate::extension::ASSEMBLY_DIRECTORY;

pub struct VirtualMachine {
    engine: Engine,
    store: Store,
    cache: ModuleCache,
    instances: Vec<Instance>
}

impl VirtualMachine {
    pub fn new() -> Result<Self> {
        let mut cranelift = Cranelift::new();
        cranelift.opt_level(CraneliftOptLevel::Speed);

        let engine = Engine::from(cranelift);
        let mut store = Store::new(&engine);
        let mut cache = ModuleCache::new()?;

        let imports = imports! {};
        let extensions = std::fs::read_dir(ASSEMBLY_DIRECTORY)?
            .filter_map(|entry| {
                // Filter out all entries that are not files with a WASM extension.

                if let Ok(dir) = entry {
                    let metadata = dir.metadata();
                    if let Err(err) = &metadata {
                        tracing::error!("Failed to read directory entry metadata: {}", err.to_string());
                    }
                    let metadata = metadata.unwrap();

                    if metadata.is_file() {
                        let file_name = dir.file_name();
                        let file_extension = Path::new(&file_name).extension().unwrap_or_default();

                        // Skip non-WebAssembly files
                        if file_extension == "wasm" {
                            let str = file_name.to_str();
                            // Check if file name is valid UTF-8.
                            if let Some(borrowed) = str {
                                return Some(borrowed.to_owned())
                            }
                        }
                    }

                    None
                } else {
                    tracing::error!("Failed to load extension directory list entry: {}", entry.unwrap_err().to_string());
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut instances = Vec::with_capacity(extensions.len());
        for ext in &extensions {
            // FIXME: Add proper error handling using a custom VmError type.

            let module = cache.load(&engine, &store, ext)?;
            let instance = Instance::new(&mut store, &module, &imports).unwrap();

            instances.push(instance);
        }
        tracing::info!("Loaded {} extension(s)", extensions.len());

        Ok(Self {
            engine, store, cache, instances
        })
    }
}