use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use wasmer::{Engine, Module, Store};
use wasmer_cache::{Cache, DeserializeError, FileSystemCache, Hash};
use util::{bail, Result};
use crate::extension::{ASSEMBLY_DIRECTORY, CACHE_DIRECTORY};

/// Manages the file system extension cache.
pub struct ModuleCache {
    fs_cache: FileSystemCache
}

impl ModuleCache {
    /// Creates a new module cache.
    pub fn new() -> Result<Self> {
        let fs_cache = FileSystemCache::new(CACHE_DIRECTORY)?;
        Ok(Self { fs_cache })
    }

    /// Loads a module from the cache.
    ///
    /// If the module was found in the cache, it will be returned immediately.
    /// If there was no precompiled module, it will be compiled, stored in the cache and returned.
    ///
    /// `name` is the name of the module. The cache will look for a file with this name in [`ASSEMBLY_DIRECTORY`].
    pub fn load<I>(&mut self, engine: &Engine, store: &Store, name: I) -> Result<Module>
    where
        I: AsRef<str>
    {
        // FIXME: Add proper error handling using a custom VmError type.

        let path = Path::new(ASSEMBLY_DIRECTORY).join(name.as_ref());

        let mut bytecode = Vec::new();
        File::open(path)?.read_to_end(&mut bytecode)?;

        // Modules are stored by hash of their contents.
        let hash = Hash::generate(&bytecode);
        let load_result = unsafe {
            self.fs_cache.load(store, hash)
        };

        if let Err(err) = load_result {
            match err {
                DeserializeError::Io(ref io) => {
                    if io.kind() == io::ErrorKind::NotFound {
                        let module = Module::new(engine, bytecode).unwrap();
                        self.fs_cache.store(hash, &module).unwrap();

                        return Ok(module)
                    }
                },
                _ => ()
            }

            bail!(Other, "Attempt to load cached extension failed: {}", err.to_string())
        } else {
            Ok(load_result.unwrap())
        }
    }
}