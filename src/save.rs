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
    #[cfg(test)]
    let dirs = ProjectDirs::from("com", "TilesRUs", "transportIO-test").unwrap();
    #[cfg(not(test))]
    let dirs = ProjectDirs::from("com", "TilesRUs", "transportIO").unwrap();
    dirs
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
    #[cfg(not(target_family = "wasm"))]
    pub fn save_exists() -> bool {
        std::fs::exists(Self::determine_save_path()).unwrap()
    }

    #[cfg(target_family = "wasm")]
    pub fn save_exists() -> bool {
        quad_storage::STORAGE.lock().unwrap().len() > 0
    }

    pub fn load() -> LoadResult {
        #[cfg(not(target_family = "wasm"))]
        let map = Self::load_desktop();
        #[cfg(target_family = "wasm")]
        let map = Self::load_wasm();

        if let Err(err) = &map {
            println!("Error loading save: {:?}", *err);
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
        Ok(std::fs::write(
            Self::determine_save_path(),
            self.to_ron_string()?,
        )?)
    }

    /// returns the save data in RON format as a pretty string
    fn to_ron_string(&self) -> Result<String, SaveError> {
        Ok(ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::default(),
        )?)
    }
}

#[cfg(test)]
mod save_tests {

    use crate::map::Direction;

    use super::*;

    #[test]
    fn test_map_serialize() {
        let _ = std::fs::remove_file(Map::determine_save_path());

        assert!(!Map::save_exists());

        let mut map = Map::new_from_string(">>>>1");

        map.add_vehicle(
            Some((map.grid.pos(0, 0), Direction::RIGHT)),
            1,
            crate::consts::SpawnerColors::Blue,
            0,
        );

        map.save().unwrap();

        assert!(Map::save_exists());

        let deserialized: Map = Map::load().unwrap();

        assert_eq!(
            deserialized,
            map
        );
    }
}
