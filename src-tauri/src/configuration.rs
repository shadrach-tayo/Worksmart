use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "worksmart", "").expect("Can't use app directory")
}

fn config_path<D>() -> PathBuf {
    let config_path = project_dirs().config_dir().to_path_buf();

    std::fs::create_dir_all(&config_path).expect("Can't create config directory");

    config_path.join(format!("{}.bin", std::any::type_name::<D>()).replace("::", "-"))
}

pub fn data_path() -> PathBuf {
    let storage_path = project_dirs().data_local_dir().to_path_buf();

    std::fs::create_dir_all(&storage_path).expect("Can't create config directory");

    storage_path
}

pub fn save<D>(data: &D)
where
    D: Serialize,
{
    let data: Vec<u8> = bincode::serialize(data).unwrap();
    std::fs::write(config_path::<D>(), data).expect("Can't save app configuration");
}

pub fn save_to_data_path<D>(data: &D, dir: PathBuf)
where
    D: Serialize,
{
    let data: Vec<u8> = bincode::serialize(data).unwrap();
    std::fs::write(data_path().join(dir), data).expect("Can't save app configuration");
}

pub fn load<D>() -> crate::Result<D>
where
    D: DeserializeOwned,
{
    let data = std::fs::read(config_path::<D>())?;
    let data: D = bincode::deserialize(&data).unwrap();

    Ok(data)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub capsule_storage_dir: PathBuf,
    pub media_storage_dir: PathBuf,
    pub preferences: Preferences,
}

impl Default for Configuration {
    fn default() -> Self {
        if let Ok(this) = load::<Self>() {
            return this;
        }

        let this = Self {
            capsule_storage_dir: PathBuf::from_str("capsules").unwrap(),
            media_storage_dir: PathBuf::from_str("media").unwrap(),
            preferences: Preferences::default(),
        };

        save(&this);

        this
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub time_gap_duration_in_seconds: u64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            time_gap_duration_in_seconds: 600,
        }
    }
}

pub type GeneralConfig = Arc<Mutex<Configuration>>;
