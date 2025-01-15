use ron::de::SpannedError;
// #[cfg(not(target_family = "wasm"))]
// use crate::dir;
// use serde::{Deserialize, Serialize};
#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

use crate::map::Map;

#[cfg(not(target_family = "wasm"))]
use directories::ProjectDirs;

#[cfg(not(target_family = "wasm"))]
/// returns the ProjectDirs struct from the directories crate with the proper identifier for the
/// game
pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("com", "yourname", "yourgame").unwrap()
}

#[cfg(not(target_family = "wasm"))]
const SAVE_FILE: &str = "save.ron";

#[cfg(target_family = "wasm")]
const WASM_SAVE_KEY: &str = "save";

#[derive(Debug)]
pub enum SaveError {
    #[allow(unused)]
    ReadFile(std::io::Error),
    #[allow(unused)]
    Deserialize(SpannedError),
    #[allow(unused)]
    Serialize(ron::Error),
}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> Self {
        SaveError::ReadFile(err)
    }
}

impl From<SpannedError> for SaveError {
    fn from(err: SpannedError) -> Self {
        SaveError::Deserialize(err)
    }
}


impl From<ron::Error> for SaveError {
    fn from(err: ron::Error) -> Self {
        SaveError::Serialize(err)
    }
}


pub type LoadResult = Result<Map, SaveError>;
type SaveResult = Result<(), SaveError>;

impl Map {

    pub fn load() -> LoadResult {
        #[cfg(not(target_family = "wasm"))]
        let mut map = Self::load_desktop();
        #[cfg(target_family = "wasm")]
        let mut map = Self::load_wasm();

        match &mut map {
            Ok(m) => m.fixup().unwrap(),
            Err(err) => println!("Error loading save: {:?}", *err),
        }

        map
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn load_desktop() -> LoadResult {
        let save_path = Self::determine_save_path();

        let toml_str = std::fs::read_to_string(save_path)?;
        Ok(ron::from_str(toml_str.as_str())?)
    }

    #[cfg(not(target_family = "wasm"))]
    fn determine_save_path() -> PathBuf {
        let project_dirs = project_dirs();
        let save_dir = project_dirs.data_local_dir();
        std::fs::create_dir_all(save_dir).unwrap();
        let mut save_path = PathBuf::from(save_dir);
        save_path.push(SAVE_FILE);
        save_path
    }

    #[cfg(target_family = "wasm")]
    pub fn load_wasm() -> LoadResult {
        let storage = &mut quad_storage::STORAGE.lock().unwrap();
        let wasm_save = storage.get(WASM_SAVE_KEY).unwrap();
        Ok(ron::from_str(wasm_save.as_str()).unwrap())
    }

    /// writes the save to local storage
    #[cfg(target_family = "wasm")]
    pub fn save(&self) -> SaveResult {
        // TODO: Solve unwrap
        let storage = &mut quad_storage::STORAGE.lock().unwrap();
        storage.set(WASM_SAVE_KEY, &self.to_ron_string().unwrap().as_str());
        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    /// writes the save to disk
    pub fn save(&self) -> SaveResult {
        Ok(std::fs::write(Self::determine_save_path(), self.to_ron_string()?)?)
    }

    /// returns the save data in RON format as a pretty string
    fn to_ron_string(&self) -> Result<String, SaveError> {
        Ok(ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?)
    }
}

#[cfg(test)]
mod save_tests {

    use super::*;

    #[test]
    #[ignore = "Overwrites current save game..."]
    fn test_map_serialize() {
        let mut map = Map::new_blank((4, 4));

        map.add_vehicle(map.grid.pos(0, 0), map.grid.pos(1, 0), crate::consts::SpawnerColors::Blue);

        map.save().unwrap();

        let mut deserialized: Map = Map::load().unwrap();

        assert_eq!(
            deserialized.grid.get_tile(&deserialized.grid.pos(0, 0)),
            map.grid.get_tile(&deserialized.grid.pos(0, 0)),
        );

        let pos = deserialized.grid.pos(0, 0);

        assert!(deserialized
            .grid
            .get_tile_mut(&pos)
            .unwrap()
            .reserve(1234, pos)
            .is_err())
    }
}
