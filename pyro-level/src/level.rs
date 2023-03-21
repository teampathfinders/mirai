// Special keys

use std::io::{Read, SeekFrom};
use std::fs::File;
use std::path::Path;
use util::{bail, Error, error, Result};
use util::bytes::{BinaryRead, BinaryWrite};
use crate::database::Database;
use crate::level_dat::LevelDat;

pub struct Level {
    pub dat: LevelDat,
    database: Database
}

impl Level {
    pub fn open<P>(path: P) -> Result<Level>
    where
        P: AsRef<Path>
    {
        let mut dat_file = File::open(path.as_ref().join("level.dat"))?;
        let mut dat_nbt = Vec::new();
        dat_file.read_to_end(&mut dat_nbt)?;

        let dat: LevelDat = nbt::from_le_bytes(&dat_nbt[8..])?.0;
        let database = Database::open(
            path
                .as_ref()
                .join("db")
                .to_str()
                .ok_or_else(|| error!(Malformed, "Invalid level path"))?
        )?;

        Ok(Level {
            dat, database
        })
    }

    pub fn flush(&self) -> Result<()> {
        todo!();
    }
}