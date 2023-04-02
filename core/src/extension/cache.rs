use std::path::{Path, PathBuf};
use wasmtime::{Engine, Module};

pub struct CompilationCache {
    dir: PathBuf
}

impl CompilationCache {
    pub fn new<I>(cache_dir: I) -> Self
    where
        I: Into<PathBuf>
    {
        Self {
            dir: cache_dir.into()
        }
    }

    pub fn load<I>(&self, engine: &Engine, file_name: I) -> anyhow::Result<Module>
    where
        I: for<'a> Into<&'a Path>
    {
        let path = self.dir.join(file_name.into());
        todo!();
    }
}