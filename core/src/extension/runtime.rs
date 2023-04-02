use std::path::{Path, PathBuf};
use anyhow::Context;
use wasmtime::{Config, Engine, Instance, Module, Store};
use crate::extension::{ASSEMBLY_DIRECTORY, Extension};

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

        let mut store = Store::new(&engine, ());
        let module = Module::from_file(&engine, "ext/rust.wasm")
            .context("Failed to compile module")?;

        let instance = Instance::new(&mut store, &module, &[])?;

        let extension = Extension::new(&instance, &mut store)?;

        todo!()
    }
}