use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use nokhwa::utils::CameraInfo;
use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub capsule_storage_dir: PathBuf,
    pub media_storage_dir: PathBuf,
    pub launch_on_startup: bool,
    pub signin_on_launch: bool,
    pub track_on_signin: bool,
    pub enable_camera: bool,
    pub preferences: Preferences,
}

impl Default for Configuration {
    fn default() -> Self {
        if let Ok(this) = storage::load::<Self>() {
            return this;
        }

        let this = Self {
            capsule_storage_dir: PathBuf::from_str("capsules").unwrap(),
            media_storage_dir: PathBuf::from_str("media").unwrap(),
            launch_on_startup: false,
            signin_on_launch: false,
            track_on_signin: false,
            enable_camera: false,
            preferences: Preferences::default(),
        };

        storage::save(&this);

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

pub type SelectedDevice = Arc<Mutex<CameraInfo>>;
