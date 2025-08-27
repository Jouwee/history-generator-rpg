use std::{fmt::Display, fs::{self, File}, path::{Path, PathBuf}, time::Instant};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{game::{chunk::{Chunk, ChunkCoord, ChunkLayer, ChunkSerialized}, state::GameState}, info, resources::resources::Resources, warn, world::world::World};

fn save_files_path() -> Result<PathBuf, LoadSaveError> {
    #[cfg(unix)]
    let app_data = std::env::var("HOME").expect("No HOME directory");
    #[cfg(windows)]
    let app_data = std::env::var("APP_DATA").expect("No APP_DATA directory");

    let directory = Path::new(&app_data).join(Path::new("Tales of Kathay"));
    if !directory.exists() {
        std::fs::create_dir(&directory)?;
    }
    
    Ok(directory)
}

pub(crate) struct SaveFile {
    save_file_name: String    
}

impl SaveFile {

    pub(crate) fn new(save_file_name: String) -> Self {
        Self {
            save_file_name,
        }
    }

    pub(crate) fn enumerate_saves() -> Result<Vec<SaveMetadata>, LoadSaveError> {
        let paths = fs::read_dir(save_files_path()?)?;
        let mut saves = Vec::new();
        for path in paths {
            let file_name = path?.file_name().into_string();
            match file_name {
                Ok(file_name) => {
                    let save = Self::new(file_name);
                    let meta = save.load_metadata();
                    if meta.is_err() {
                        continue;
                    }
                    saves.push(meta?);
                },
                Err(_) => warn!("Can't enumerate save file"),
            }
        }
        return Ok(saves);
    }

    pub(crate) fn create_new_save_file() -> Result<SaveFile, LoadSaveError> {
        let mut last_save = 1;
        for save in Self::enumerate_saves()? {
            let number: i32 = save.save_file_name.split("_").last().unwrap().parse().unwrap();
            if number >= last_save {
                last_save = number + 1;
            }
        }
        return Ok(Self::new(format!("save_{last_save}")));
    }

    pub(crate) fn save_world(&self, world: &World) -> Result<(), LoadSaveError> {
        let timing = Instant::now();

        let mut metadata = self.load_or_create_metadata()?;
        metadata.last_played = Local::now();
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

    pub(crate) fn save_game_state(&self, state: &GameState) -> Result<(), LoadSaveError> {
        let timing = Instant::now();

        let mut metadata = self.load_or_create_metadata()?;
        metadata.last_played = Local::now();
        self.save_metadata(&metadata)?;

        let buffer = File::create(self.path("state")?)?;
        ciborium::into_writer(&state, buffer)?;

        info!("save_game_state took {:.2?}", timing.elapsed());

        return Ok(())
    }

    pub(crate) fn load_game_state(&self) -> Result<GameState, LoadSaveError> {
        let timing = Instant::now();

        let buffer = File::open(self.path("state")?)?;
        let state = ciborium::from_reader(buffer)?;

        info!("load_game_state took {:.2?}", timing.elapsed());

        return Ok(state);
    }

    pub(crate) fn save_chunk(&self, chunk: &Chunk) -> Result<(), LoadSaveError> {
        let timing = Instant::now();

        let mut metadata = self.load_or_create_metadata()?;
        metadata.last_played = Local::now();
        self.save_metadata(&metadata)?;

        let buffer = File::create(self.chunk_path(&chunk.coord)?)?;
        let chunk = ChunkSerialized::from_chunk(chunk);
        ciborium::into_writer(&chunk, buffer)?;

        info!("save_chunk took {:.2?}", timing.elapsed());

        return Ok(())
    }

    pub(crate) fn load_chunk(&self, coord: &ChunkCoord, resources: &Resources) -> Result<Chunk, LoadSaveError> {
        let timing = Instant::now();

        let buffer = File::open(self.chunk_path(coord)?)?;
        let chunk: ChunkSerialized = ciborium::from_reader(buffer)?;
        let chunk = chunk.to_chunk(resources);

        info!("load_chunk took {:.2?}", timing.elapsed());

        return Ok(chunk);
    }

    fn chunk_path(&self, coord: &ChunkCoord) -> Result<PathBuf, LoadSaveError> {
        let layer = match coord.layer {
            ChunkLayer::Surface => "surface",
            ChunkLayer::Underground => "underground",
        };
        
        let folder = self.path("chunks")?;
        if !folder.exists() {
            std::fs::create_dir(&folder)?;
        }

        Ok(folder.join(&format!("{}.{}.{}", coord.xy.x, coord.xy.y, layer)))
    } 

    fn load_or_create_metadata(&self) -> Result<SaveMetadata, LoadSaveError> {
        if !Path::new(&self.path("savefile")?).exists() {
            self.save_metadata(&SaveMetadata {
                save_name: self.save_file_name.clone(),
                save_file_name: self.save_file_name.clone(),
                save_version: String::from(env!("CARGO_PKG_VERSION")),
                created: Local::now(),
                last_played: Local::now(),
            })?;
        }
        self.load_metadata()
    }

    pub(crate) fn load_metadata(&self) -> Result<SaveMetadata, LoadSaveError> {
        let buffer = File::open(self.path("savefile")?)?;
        let metadata = ciborium::from_reader(buffer)?;
        return Ok(metadata)
    }

    fn save_metadata(&self, metadata: &SaveMetadata) -> Result<(), LoadSaveError> {
        let buffer = File::create(self.path("savefile")?)?;
        ciborium::into_writer(&metadata, buffer)?;
        return Ok(())
    }

    fn path(&self, suffix: &str) -> Result<PathBuf, LoadSaveError> {
        let root_dir = save_files_path()?;
        if !root_dir.exists() {
            std::fs::create_dir(&root_dir)?;
        }

        let save_dir = root_dir.join(Path::new(&self.save_file_name));
        if !save_dir.exists() {
            std::fs::create_dir(&save_dir)?;
        }

        Ok(save_dir.join(Path::new(suffix)))
    }

}

/// IMPORTANT
/// Avoid changing this struct, as it's not easily versionable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SaveMetadata {
    pub(crate) save_name: String,
    pub(crate) save_file_name: String,
    pub(crate) save_version: String,
    pub(crate) created: DateTime<Local>,
    pub(crate) last_played: DateTime<Local>,
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
