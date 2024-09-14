use std::path::PathBuf;

use tauri::AppHandle;
use tokio::fs;

use crate::Result;

pub async fn get_storage_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let data_path = tauri::api::path::app_data_dir(&app_handle.config()).unwrap_or_default();
    let storage_path = data_path.join("media");
    let path = storage_path.to_str().unwrap();

    println!("Data path: {:?}", &data_path);
    let exists = tokio::fs::try_exists(data_path.clone().to_str().unwrap()).await?;
    if !exists {
        match fs::create_dir(&data_path).await {
            Ok(()) => {}
            Err(err) => println!("Error creating data folder: {:?}", err),
        }
    }

    println!("Storage path: {:?}", &storage_path);
    let exists = tokio::fs::try_exists(path).await?;
    if !exists {
        match fs::create_dir(&storage_path).await {
            Ok(()) => {}
            Err(err) => println!("Error creating storage folder: {:?}", err),
        }
    }

    Ok(storage_path)
}
