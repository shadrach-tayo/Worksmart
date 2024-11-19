use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

use tauri::AppHandle;
// use tokio::fs;
use std::fs;

use crate::Result;

pub fn get_storage_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let data_path = tauri::api::path::app_data_dir(&app_handle.config()).unwrap_or_default();
    let storage_path = data_path.join("media");
    let path = storage_path.to_str().unwrap();

    println!("Data path: {:?}", &data_path);
    let exists = fs::metadata(data_path.clone().to_str().unwrap()).is_ok();
    if !exists {
        match fs::create_dir(&data_path) {
            Ok(()) => {}
            Err(err) => println!("Error creating data folder: {:?}", err),
        }
    }

    println!("Storage path: {:?}", &storage_path);
    let exists = fs::metadata(path).is_ok();
    if !exists {
        match fs::create_dir(&storage_path) {
            Ok(()) => {}
            Err(err) => println!("Error creating storage folder: {:?}", err),
        }
    }

    Ok(storage_path)
}

pub fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "worksmart", "").expect("Can't use app directory")
}

pub fn config_path<D>() -> PathBuf {
    let config_path = project_dirs().config_dir().to_path_buf();

    std::fs::create_dir_all(&config_path).expect("Can't create config directory");

    config_path.join(format!("{}.bin", std::any::type_name::<D>()).replace("::", "-"))
}

pub fn auth_path<D>() -> PathBuf {
    let auth_path = project_dirs().data_local_dir().to_path_buf();

    std::fs::create_dir_all(&auth_path).expect("Can't create config directory");

    auth_path.join(format!("{}.bin", std::any::type_name::<D>()))
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

pub fn save_to_path<D>(data: &D, path: PathBuf) -> crate::Result<()>
where
    D: Serialize,
{
    let data: Vec<u8> = bincode::serialize(data).unwrap();
    std::fs::write(&path, data).map_err(|err| err.to_string())?;
    Ok(())
}

pub fn save_to_data_path<D>(data: &D, dir: PathBuf)
where
    D: Serialize,
{
    let data = serde_json::json!(data);
    let mut bytes: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut bytes, &data).unwrap();
    std::fs::write(data_path().join(dir), bytes).expect("Can't save app configuration");
}

pub fn load<D>() -> crate::Result<D>
where
    D: DeserializeOwned,
{
    let data = std::fs::read(config_path::<D>())?;
    let data: D = bincode::deserialize(&data).unwrap();

    Ok(data)
}

pub fn load_from_path<D>(path: PathBuf) -> crate::Result<D>
where
    D: DeserializeOwned,
{
    let data = std::fs::read(path)?;
    let data: D = bincode::deserialize(&data).unwrap();

    Ok(data)
}
