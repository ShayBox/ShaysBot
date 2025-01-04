use std::{
    collections::HashMap,
    fs::File,
    io::{ErrorKind, Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use azalea::{
    app::{App, Plugin},
    prelude::*,
    BlockPos,
};
use derive_new::new as New;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use uuid::Uuid;

#[serde_as]
#[derive(Clone, Default, Deserialize, Serialize, New)]
#[serde(default)]
pub struct StasisChamber {
    #[serde_as(as = "DisplayFromStr")]
    pub block_pos:  BlockPos,
    pub entity_id:  u32,
    pub owner_uuid: Uuid,
    pub location:   String,
}

#[serde_as]
#[derive(Clone, Default, Deserialize, Serialize, Resource)]
pub struct StasisChambers(#[serde_as(as = "Vec<(_, _)>")] pub HashMap<Uuid, StasisChamber>);

impl StasisChambers {
    /// # Errors
    /// Will return `Err` if `std::env::current_exe` fails.
    pub fn path() -> Result<PathBuf> {
        let mut path = std::env::current_exe()?;
        path.set_file_name("stasis-chambers");
        path.set_extension("yaml");

        Ok(path)
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `toml::to_string_pretty`, or `File::write_all` fails.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                Self::default().save()?;
                File::open(&path)?
            }
            Err(error) => bail!(error),
        };

        let mut text = String::new();
        file.read_to_string(&mut text)?;
        file.rewind()?;

        Ok(serde_yml::from_str(&text)?)
    }

    /// # Errors
    /// Will return `Err` if `File::open`, `File::read_to_string`, `File::rewind`, or `toml::from_str` fails.
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let text = serde_yml::to_string(&self)?;
        let buf = text.as_bytes();
        file.write_all(buf)?;

        Ok(())
    }
}

/// Handle global stasis chambers.
pub struct StasisChambersPlugin;

impl Plugin for StasisChambersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StasisChambers::load().expect("Failed to load stasis chambers"));
    }
}
