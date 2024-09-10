// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core_graphics::access::ScreenCaptureAccess;
use std::{
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tokio::fs;
use xcap::{Monitor, Window as XcapWindow};

#[allow(clippy::default_constructed_unit_structs)]
fn has_permission() -> bool {
    ScreenCaptureAccess::default().preflight()
}

#[allow(clippy::default_constructed_unit_structs)]
fn request_permission() -> bool {
    ScreenCaptureAccess::default().request()
}

use worksmart::Result;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn normalized(filename: &str) -> String {
    filename
        .replace("|", "")
        .replace("\\", "")
        .replace(":", "")
        .replace("/", "")
}

#[tauri::command]
async fn capture_screen(window: Window) {
    // MacOs check permission
    if !has_permission() {
        println!("No Graphics access");
        let granted = request_permission();
        println!("Graphics access granted: {granted}");
    }

    let app_handle = window.app_handle();
    let storage_path = get_storage_path(app_handle).await;
    if storage_path.is_ok() {
        let start = Instant::now();
        let monitors = Monitor::all().unwrap();
        let storage_path = storage_path.unwrap();
        for monitor in monitors {
            println!("Monitor: {:?}", monitor.name());
            let image = monitor.capture_image().unwrap();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let img_path = storage_path.clone().join(format!(
                "{}-{}.png",
                timestamp.as_nanos(),
                monitor.name(),
            ));

            let file = fs::File::create(img_path.clone()).await;
            println!("PreSave: {:?}, Exists: {}", &img_path, file.is_ok());

            if file.is_ok() {
                println!("Save: {:?}", &img_path);
                image.save(img_path).unwrap();
            }
        }

        println!("Done: {:?}", start.elapsed());

        let windows = XcapWindow::all().unwrap();

        let start = Instant::now();
        for window in windows {
            // if window.is_minimized() {
            //     continue;
            // }

            println!(
                "Window: {:?} {:?} {:?}",
                window.title(),
                (window.x(), window.y(), window.width(), window.height()),
                (window.is_minimized(), window.is_maximized())
            );

            let image = window.capture_image().unwrap();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let img_path = storage_path.clone().join(format!(
                "{}-{}.png",
                timestamp.as_nanos(),
                normalized(window.title()),
            ));

            let file = fs::File::create(img_path.clone()).await;
            println!("PreSave: {:?}, Exists: {}", &img_path, file.is_ok());
            image.save(img_path).unwrap();

            // i += 1;
        }
        println!("Windows Done: {:?}", start.elapsed());
    }
}

async fn get_storage_path(app_handle: AppHandle) -> Result<PathBuf> {
    let data_path = tauri::api::path::app_data_dir(&app_handle.config()).unwrap_or_default();
    let storage_path = data_path.join("captures");
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

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, capture_screen])
        .on_window_event(|event| match event.event() {
            WindowEvent::Focused(focused) => {
                if !focused {
                    // event.window().hide().unwrap();
                    println!("Focused: {focused}!");
                }
            }
            WindowEvent::CloseRequested { api, .. } => {
                println!("close request!");
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        });

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
