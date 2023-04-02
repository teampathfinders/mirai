// Special keys

use crate::database::Database;
use crate::level_dat::LevelDat;
use anyhow::anyhow;
use std::fs::File;
use std::io::{Read, SeekFrom};
use std::path::Path;
use util::bytes::{BinaryRead, BinaryWrite};
use util::{bail, error, Error, Result};

pub struct Level {
    pub dat: LevelDat,
    database: Database,
}

impl Level {
    pub fn open<P>(path: P) -> anyhow::Result<Level>
    where
        P: AsRef<Path>,
    {
        let mut dat_file = File::open(path.as_ref().join("level.dat"))?;
        let mut dat_nbt = Vec::new();
        dat_file.read_to_end(&mut dat_nbt)?;

        let dat: LevelDat = nbt::from_le_bytes(&dat_nbt[8..])?.0;
        let database = Database::open(path.as_ref().join("db").to_str().ok_or_else(|| anyhow!("Invalid level path"))?)?;

        Ok(Level { dat, database })
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        todo!();
    }
}
