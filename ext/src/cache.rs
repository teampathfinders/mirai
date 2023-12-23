use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use wasmtime::{Engine, Module};

use super::ASSEMBLY_DIRECTORY;

/// Manages a filesystem cache of compiled extensions.
///
/// Each extension is stored as a separate file in the cache directory.
/// The name of the cache file corresponds to a SHA-256 hash of the extension's contents.
/// Additionally, each cache file is also compressed with Zlib.
pub struct CompilationCache {
    /// Directory the cache is located in.
    cache_dir: PathBuf,
}

impl CompilationCache {
    /// Initialises a new compilation cache.
    ///
    /// This function creates a directory at the location specified in `cache_dir`
    /// if it does not exist.
    ///
    /// # Errors
    ///
    /// An error will be returned if any of the file operations fail.
    pub fn new<P>(cache_dir: P) -> anyhow::Result<Self>
    where
        P: Into<PathBuf>,
    {
        let cache_dir = cache_dir.into();
        if !cache_dir.try_exists().context("Could not verify that cache directory exists")? {
            std::fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self { cache_dir })
    }

    /// Attempts to load a cached extension.
    ///
    /// If a cache entry does not exist, the extension will first be compiled and then cached.
    ///
    /// # Errors
    ///
    /// This function will return an error if filesystem I/O operations fail,
    /// if the file is an invalid Zlib stream or if the extension is malformed.
    pub fn load(&self, engine: &Engine, file_name: &str) -> anyhow::Result<Module> {
        let assembly_path = Path::new(ASSEMBLY_DIRECTORY).join(file_name);
        let mut bytecode = Vec::new();
        File::open(&assembly_path)
            .context(format!("Could not find extension assembly {}", assembly_path.display()))?
            .read_to_end(&mut bytecode)?;

        // Cache names are SHA-256 hashes of the extension's source.
        let mut hasher = Sha256::new();
        hasher.update(&bytecode);
        let hash = hasher.finalize();
        let hash_string = format!("{hash:x}");

        let cache_path = self.cache_dir.join(hash_string);

        // Load the cache file if it exists...
        if cache_path.try_exists()? {
            let cache_file = BufReader::new(File::open(cache_path)?);
            let mut decoder = ZlibDecoder::new(cache_file);

            let mut cache_bytecode = Vec::new();
            decoder.read_to_end(&mut cache_bytecode)?;

            // Load cache
            Ok(unsafe { Module::deserialize(engine, cache_bytecode)? })
        }
        // ...and compile it if it doesn't.
        else {
            let extension = Module::new(engine, bytecode)?;
            let serialized = extension.serialize()?;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(&serialized)?;
            let compressed = encoder.finish()?;

            File::create(cache_path)?.write_all(&compressed)?;

            Ok(extension)
        }
    }
}
