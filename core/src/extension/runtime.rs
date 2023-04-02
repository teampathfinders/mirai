use std::path::{Path, PathBuf};
use wasmtime::{Config, Engine, Instance, Module, Store};
use crate::extension::{ASSEMBLY_DIRECTORY, Extension};

pub struct Runtime {
    engine: Engine,
    store: Store<()>,
    modules: Vec<Extension>
}

impl Runtime {
    pub fn new() -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.parallel_compilation(true);

        let engine = Engine::new(&config)?;
        let mut store = Store::new(&engine, ());

        let module = Module::from_file(&engine, "ext/rust.wasm")?;
        let instance = Instance::new(&mut store, &module, &[])?;

        let extension = Extension::new(&instance, &mut store)?;

        todo!()
    }
}