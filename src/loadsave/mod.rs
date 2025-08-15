use std::{fmt::Display, fs::File, path::{Path, PathBuf}, time::Instant};

use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};

use crate::{info, world::world::World};

pub(crate) struct LoadSaveManager {
}

impl LoadSaveManager {

    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn save_world(&self, world: &World) -> Result<(), LoadSaveError> {
        let timing = Instant::now();

        let mut metadata = self.load_or_create_metadata()?;
        metadata.last_played = Local::now();
        // TODO(ROO4JcDl): Playtime
        self.save_metadata(&metadata)?;

        let buffer = File::create(self.path("world")?)?;
        ciborium::into_writer(&world, buffer)?;

        info!("save_world took {:.2?}", timing.elapsed());

        return Ok(())
    }

    pub(crate) fn load_world(&self) -> Result<World, LoadSaveError> {
        let timing = Instant::now();

        let buffer = File::open(self.path("world")?)?;
        let world = ciborium::from_reader(buffer)?;

        info!("load_world took {:.2?}", timing.elapsed());

        return Ok(world);
    }

    fn load_or_create_metadata(&self) -> Result<SaveMetadata, LoadSaveError> {
        if !Path::new(&self.path("savefile")?).exists() {
            self.save_metadata(&SaveMetadata {
                save_version: String::from(env!("CARGO_PKG_VERSION")),
                created: Local::now(),
                last_played: Local::now(),
                playtime: Local::now() - Local::now(),
            })?;
        }
        self.load_metadata()
    }

    fn load_metadata(&self) -> Result<SaveMetadata, LoadSaveError> {
        let buffer = File::open(self.path("savefile")?)?;
        // TODO(ROO4JcDl): Check: If you want to deserialize faster at the cost of more memory, consider using from_reader_with_buffer with a larger buffer, for example 64KB.
        let metadata = ciborium::from_reader(buffer)?;
        return Ok(metadata)
    }

    fn save_metadata(&self, metadata: &SaveMetadata) -> Result<(), LoadSaveError> {
        let buffer = File::create(self.path("savefile")?)?;
        ciborium::into_writer(&metadata, buffer)?;
        return Ok(())
    }

    fn path(&self, suffix: &str) -> Result<PathBuf, LoadSaveError> {
        #[cfg(unix)]
        let app_data = std::env::var("HOME").expect("No HOME directory");
        #[cfg(windows)]
        let app_data = std::env::var("APP_DATA").expect("No APP_DATA directory");

        let directory = Path::new(&app_data).join(Path::new("Tales of Kathay"));
        if !directory.exists() {
            std::fs::create_dir(&directory)?;
        }
        
        Ok(directory.join(Path::new(suffix)))
    }

}

/// IMPORTANT
/// Avoid changing this struct, as it's not easily versionable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SaveMetadata {
    pub(crate) save_version: String,
    pub(crate) created: DateTime<Local>,
    pub(crate) last_played: DateTime<Local>,
    pub(crate) playtime: Duration,
    // TODO(ROO4JcDl):
    // Played creatures
}

#[derive(Debug)]
pub(crate) struct LoadSaveError {
    message: String,
}

impl Display for LoadSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(&self.message);
    }
}

impl From<std::io::Error> for LoadSaveError {
    fn from(value: std::io::Error) -> Self {
        Self { message: value.to_string() }
    }
}

impl From<ciborium::ser::Error<std::io::Error>> for LoadSaveError {
    fn from(value: ciborium::ser::Error<std::io::Error>) -> Self {
        Self { message: value.to_string() }
    }
}

impl From<ciborium::de::Error<std::io::Error>> for LoadSaveError {
    fn from(value: ciborium::de::Error<std::io::Error>) -> Self {
        Self { message: value.to_string() }
    }
}
