use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context};
use flate2::Compression;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha2::{Sha256, Digest};
use wasmtime::{Engine, Module};

use super::ASSEMBLY_DIRECTORY;

pub struct CompilationCache {
    cache_dir: PathBuf
}

impl CompilationCache {
    pub fn new<P>(cache_dir: P) -> anyhow::Result<Self>
    where
        P: Into<PathBuf>
    {
        let cache_dir = cache_dir.into();
        if !cache_dir.try_exists()
            .context("Could not verify that cache directory exists")? 
        {
            std::fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            cache_dir
        })
    }

    pub fn load(&self, engine: &Engine, file_name: &str) -> anyhow::Result<Module> {    
        let assembly_path = Path::new(ASSEMBLY_DIRECTORY).join(file_name);
        let mut bytecode = Vec::new();
        File::open(&assembly_path)
            .context(format!("Could not find extension assembly {}", assembly_path.display()))?
            .read_to_end(&mut bytecode)?;

        let mut hasher = Sha256::new();
        hasher.update(&bytecode);
        let hash = hasher.finalize();
        let hash_string = format!("{hash:x}");

        let cache_path = self.cache_dir.join(hash_string);
        if cache_path.try_exists()? {
            tracing::info!("Loading cached '{file_name}' module");

            let cache_file = BufReader::new(File::open(cache_path)?);
            let mut decoder = ZlibDecoder::new(cache_file);
            
            let mut cache_bytecode = Vec::new();
            decoder.read_to_end(&mut cache_bytecode)?;

            // Load cache
            Ok(unsafe {
                Module::deserialize(engine, cache_bytecode)?
            })
        } else {
            tracing::info!("Precompiling extension module '{file_name}'");

            let module = Module::new(engine, bytecode)?;
            let serialized = module.serialize()?;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(&serialized)?;
            let compressed = encoder.finish()?;

            File::create(cache_path)?.write_all(&compressed)?;

            Ok(module)
        }
    }
}