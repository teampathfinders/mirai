use std::fmt::{self, Debug, Formatter};

use crate::stdio::ExtensionStdout;
use anyhow::{anyhow, Context};
use wasmtime::{Instance, Store, TypedFunc};
use wasmtime_wasi::stdio::stdout;
use wasmtime_wasi::WasiCtx;

use crate::def;

/// Contains commonly used functions.
struct PluginFns {
    /// Allocates a block of memory in WebAssembly linear memory.
    ///
    /// The argument passed into this function determines the size of the allocation in bytes.
    alloc_fn: TypedFunc<u32, i32>,
    /// Deallocates a block of memory previously allocated with [`alloc_fn`](Self::alloc_fn).
    ///
    /// The first argument is a pointer to the start of the allocation while the second argument is the
    /// size of the allocation.
    /// The size passed into this function must be equal to the size previously used in [`alloc_fn`](Self::alloc_fn).
    dealloc_fn: TypedFunc<(i32, u32), ()>,
    /// Resizes a block of memory previously allocated with [`alloc_fn`](Self::alloc_fn).
    ///
    /// The first argument is a pointer to the start of the allocation,
    /// the second argument is the current size of the allocation,
    /// and the third argument is the requested size of the new allocation.
    realloc_fn: TypedFunc<(i32, u32, u32), i32>,
}

#[inline]
fn to_version_string(version: [u8; 4]) -> String {
    // Iterator::intersperse is unstable, so this is a manual implementation.
    let mut version = version.iter().map(|n| n.to_string() + ".").collect::<String>();
    version.truncate(version.len() - 1);
    version
}

/// A WebAssembly server plugin.
pub struct Plugin {
    /// Name of the plugin
    ///
    /// This name is retrieved using the `_pyro_ext_name` and `__pyro_ext_name_len` exports.
    name: String,
    /// Name of the WebAssemby file.
    file: String,
    /// Version of this plugin.
    ///
    /// Retrieved using the `__pyro_ext_version` export.
    version: [u8; 4],
    /// The instantiated plugin.
    instance: Instance,
    store: Store<WasiCtx>,
    /// Pointers to several functions exported by all plugins.
    fn_pointers: PluginFns,
}

impl Plugin {
    /// Creates a new plugin using the given instance and store.
    ///
    /// # Initialisation
    ///
    /// This function loads all the required function exports and then loads the plugin name and version.
    ///
    /// Firstly, the `__pyro_initialize` function runs, which should be used to initialise runtime resources
    /// such as a garbage collector.
    ///
    /// Secondly, the `__pyro_ext_name_len` function is called, which returns the length of the name in bytes.
    /// The runtime then allocates a buffer of the specified size using `__pyro_alloc`,
    /// which is populated using `__pyro_ext_name`.
    /// This UTF-8 buffer is converted into a Rust string and then deallocated with `__pyro_dealloc`.
    ///
    /// Lastly, the plugin version is simply retrieved using `__pyro_ext_version`.
    /// The version is encoded as an unsigned little-endian 32-bit integer. Each of the bytes in the integer
    /// is a component of the 4-component Semver version.
    ///
    /// # Cleanup
    ///
    /// The server will call the `__pyro_cleanup` function to cleanup remaining runtime resources.
    /// This happens after most of the server has already shut down and should therefore not interact with the server
    /// in any way.
    ///
    /// # Errors
    ///
    /// This function can fail if a required export is not found or if the given name
    /// is not valid UTF-8.
    ///
    /// # Panics
    ///
    /// This function panics if the instance was not created with the specified store.
    pub fn new(file: String, instance: Instance, mut store: Store<WasiCtx>) -> anyhow::Result<Self> {
        {
            // Run __pyro_initialize if it exists.
            // This function is used to initialise the language runtime before doing anything else.
            // AssemblyScript requires this or the majority of functions will abort the plugin.
            if let Some(preinit) = instance.get_func(&mut store, def::IMPL_INITIALIZE_FN) {
                let preinit = preinit.typed::<(), ()>(&mut store)?;
                preinit.call(&mut store, ())?;
            }
        }

        let alloc_fn = instance.get_typed_func::<u32, i32>(&mut store, def::IMPL_ALLOC_FN)?;
        let dealloc_fn = instance.get_typed_func::<(i32, u32), ()>(&mut store, def::IMPL_DEALLOC_FN)?;
        let realloc_fn = instance.get_typed_func::<(i32, u32, u32), i32>(&mut store, def::IMPL_REALLOC_FN)?;

        let name = {
            let name_len_fn = instance.get_typed_func::<(), u32>(&mut store, def::IMPL_EXT_NAME_LEN_FN)?;
            let len = name_len_fn.call(&mut store, ())?;

            let name_fn = instance.get_typed_func::<i32, ()>(&mut store, def::IMPL_EXT_NAME_FN)?;
            let name_ptr = alloc_fn.call(&mut store, len)?;
            name_fn.call(&mut store, name_ptr)?;

            let linear_mem = instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| anyhow!("Memory export 'memory' not found").context("Failed to load plugin name"))?;

            let mut utf8_buffer = vec![0; len as usize];
            linear_mem.read(&mut store, name_ptr as usize, &mut utf8_buffer)?;

            let name = String::from_utf8(utf8_buffer).context("Failed to read plugin name")?;

            dealloc_fn.call(&mut store, (name_ptr, len))?;

            name
        };

        let version = {
            let version_fn = instance.get_typed_func::<(), u32>(&mut store, def::IMPL_EXT_VERSION_FN)?;
            version_fn.call(&mut store, ())?.to_le_bytes()
        };

        store
            .data_mut()
            .set_stdout(Box::new(ExtensionStdout { prefix: name.clone(), stdout: stdout() }));

        Ok(Self {
            name,
            file,
            version,
            instance,
            store,
            fn_pointers: PluginFns { alloc_fn, dealloc_fn, realloc_fn },
        })
    }

    /// Calls the optional startup function in the plugin.
    pub fn on_startup(&mut self) -> anyhow::Result<()> {
        if let Some(untyped) = self.instance.get_func(&mut self.store, def::ENABLE_FN) {
            let typed = untyped.typed::<(), ()>(&mut self.store)?;
            typed.call(&mut self.store, ())?;
        }

        Ok(())
    }

    /// Calls the optional shutdown function in the plugin and drops it.
    pub fn on_shutdown(mut self) -> anyhow::Result<()> {
        if let Some(untyped) = self.instance.get_func(&mut self.store, def::DISABLE_FN) {
            let typed = untyped.typed::<(), ()>(&mut self.store)?;
            typed.call(&mut self.store, ())?;
        }

        Ok(())
    }

    /// Returns the name of this plugin.
    ///
    /// See [`new`](Self::new) for an explanation of how this name is determined.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the version of this plugin.
    ///
    /// See [`new`](Self::new) for an explanation of how this version is determined.
    #[inline]
    pub const fn version(&self) -> [u8; 4] {
        self.version
    }

    /// Returns the version as a string in x.y.z format.
    #[inline]
    pub fn version_string(&self) -> String {
        to_version_string(self.version)
    }
}

impl Debug for Plugin {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Extension")
            .field("name", &self.name)
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        // Run cleanup function if it exists.
        if let Some(disable_fn) = self.instance.get_func(&mut self.store, def::IMPL_CLEANUP_FN) {
            if let Ok(disable_fn) = disable_fn.typed::<(), ()>(&mut self.store) {
                if let Err(err) = disable_fn.call(&mut self.store, ()) {
                    tracing::error!("Failed to call `{}` in module `{}@{}`: {err}", def::IMPL_CLEANUP_FN, self.name, self.file);
                }
            } else {
                tracing::error!("`{}` in module `{}@{}` has invalid signature", def::IMPL_CLEANUP_FN, self.name, self.file);
            }
        }
    }
}
