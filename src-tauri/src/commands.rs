#![allow(unused_imports)]

use chrono::Utc;
use core_graphics::access::ScreenCaptureAccess;

use dcv_color_primitives::convert_image;
// use gst::prelude::*;
use image::{ImageBuffer, ImageReader, Rgb, RgbImage};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    yuyv422_to_rgb, CameraFormat, CameraIndex, CameraInfo, FrameFormat, RequestedFormat,
    RequestedFormatType, Resolution,
};
use nokhwa::{backends::capture::*, Camera};
use nokhwa::{native_api_backend, pixel_format, NokhwaError};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::Permissions;
use std::io::{Cursor, Read};
use std::sync::atomic::Ordering;
use std::{
    io::Write,
    ops::Mul,
    path::PathBuf,
    sync::{
        atomic::{self, AtomicBool, AtomicUsize},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, State, Window};
use tokio::{fs, sync::broadcast, time};
#[allow(unused_imports)]
use xcap::{Monitor, Window as XcapWindow};
use yuv::convert::ToRGB;

use crate::session::SessionControllerState;
use crate::time_map::{TimeTrackerMap, TrackHistory};
// use crate::encoder::uyvy422_frame;
use crate::{path_exists, storage, windows, AppWindow, Auth, AuthConfig, CameraController, Configuration, PermisssionsStatus, SelectedDevice};

use crate::{
    configuration, gen_rand_string, get_current_datetime,
    session::{SessionChannel, SessionState},
    AppState, GeneralConfig, RecordChannel, Session, Shutdown,
};

use scap::capturer::{Area, Capturer, Options, Point, Size};

#[tauri::command]
pub async fn record_screen(
    // window: Window,
    record_channel: State<'_, RecordChannel>,
) -> Result<(), ()> {
    let supported = scap::is_supported();
    if !supported {
        eprintln!("Platform is not supported ❌");
    } else {
        println!("✅ Platform is supported");
    }

    if !scap::has_permission() {
        eprintln!("❌ Permission not granted. Requesting permission...");
        if !scap::request_permission() {
            println!("❌ Permission denied");
            // return Err("Permission denied".into());
            return Err(());
        }
    }

    println!("✅ Permission granted");

    record_channel
        .try_send(crate::RecordCommand::Start)
        .expect("Can't start recording");

    tokio::time::sleep(Duration::from_secs(10)).await;

    println!("Send stop siganl");

    record_channel
        .try_send(crate::RecordCommand::Stop)
        .expect("Can't start recording");

    Ok(())
}

#[allow(dead_code)]
fn normalized(filename: &str) -> String {
    filename
        .replace("|", "")
        .replace("\\", "")
        .replace(":", "")
        .replace("/", "")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[tauri::command]
pub async fn start_session(
    window: Window,
    session_rx: State<'_, SessionChannel>,
    session: State<'_, SessionState>,
    session_controller: State<'_, SessionControllerState>,
) -> Result<Option<SessionDetail>, ()> {
    let sesh = session.lock().unwrap().clone();
    if sesh.is_running {
        return Ok(None);
    }

    let app_handle = window.app_handle();

    let id = gen_rand_string(16);
    let started_at = get_current_datetime().to_rfc3339();

    *session.lock().unwrap() = Session {
        id: id.clone(),
        started_at: Some(started_at.clone()),
        is_running: true,
        ended_at: None,
        notify_shutdown: session_rx.inner().to_owned(),
        shutdown: Arc::new(Shutdown::new(session_rx.subscribe())),
    };

    session_controller.lock().unwrap().start();

    let active_session = session.lock().unwrap().clone();

    let handle = app_handle.clone();
    tokio::spawn(async move {
        if let Err(err) = active_session.start(app_handle).await {
            println!("Session Error: {:?}", err);
        }
        handle.state::<SessionState>().lock().unwrap().is_running = false;
        handle.state::<SessionState>().lock().unwrap().ended_at = Some(get_current_datetime().to_rfc3339());
        handle.emit_all("SessionEnded", ()).unwrap();
        println!("Close session#start thread");
    });

    Ok(Some(SessionDetail {
        id,
        started_at,
        ended_at: None,
    }))
}

use tauri::api::dialog::blocking::{confirm, ask};
#[tauri::command]
pub async fn stop_session(
    window: Window,
    session: State<'_, SessionState>,
    session_rx: State<'_, SessionChannel>,
    session_controller: State<'_, SessionControllerState>,
) -> Result<(), ()> {
    if !session.lock().unwrap().is_running {
        return Ok(());
    }

    let end_after_current_session = ask(Some(&window), "End session", "Do you want to end this session after this time gap?");

    if end_after_current_session {
        session_controller.lock().unwrap().shutdown();
        dbg!(session_controller.lock().unwrap().is_shutdown());
    } else {
        session_rx.send(()).unwrap();
    }

    Ok(())
}

#[tauri::command]
pub async fn get_session(session: State<'_, SessionState>) -> Result<Option<SessionDetail>, ()> {
    if session.lock().unwrap().started_at.is_none() {
        return Ok(None);
    }

    let Session {
        id,
        started_at,
        ended_at,
        ..
    } = session.lock().unwrap().clone();
    Ok(Some(SessionDetail {
        id,
        ended_at,
        started_at: started_at.unwrap(),
    }))
}

#[tauri::command]
pub fn get_permission_status() -> PermisssionsStatus {
    PermisssionsStatus::get_status()
}

#[tauri::command]
pub fn request_camera_permissions() {
    PermisssionsStatus::request_permission(crate::PermissionType::Camera);
}

#[tauri::command]
pub fn request_accessibility_permissions() {
    PermisssionsStatus::request_permission(crate::PermissionType::Accessibility);
}

#[tauri::command]
pub fn request_screen_capture_permissions() {
    PermisssionsStatus::request_permission(crate::PermissionType::ScreenCapture);
}

#[tauri::command]
pub fn on_permissions_granted(window: Window, auth: State<'_, AuthConfig>) {
    (AppWindow::Permissions {}).close(&window.app_handle());

    let handle = window.app_handle();

    if auth.lock().unwrap().is_none() {
        windows::show_login(&handle);
    } else {
        windows::show_tracker(&handle);
    }
}

#[tauri::command]
pub fn set_preferences(
    general_config: State<'_, GeneralConfig>,
    preferences: Configuration,
) -> Result<(), String> {
     dbg!(&preferences);
    *general_config.lock().unwrap() = preferences;

    storage::save(&general_config.lock().unwrap().clone());

    Ok(())
}

#[tauri::command]
pub fn get_preferences(general_config: State<'_, GeneralConfig>) -> Result<Configuration, String> {
    let config = general_config.lock().unwrap().clone();
    dbg!(&config);
    Ok(config)
}

#[tauri::command]
pub fn get_auth(auth_config: State<'_, AuthConfig>) -> Result<Option<Auth>, String> {
    // ideally take general config struct from client and save first
    // then assign it as new general_config: config_state.lock().unwrap() = configuration

    let auth = auth_config.lock().unwrap().clone();

    println!("Get auth: {:?}", &auth);

    Ok(auth)
}


#[tauri::command]
pub async fn webcam_capture(
    general_config: State<'_, GeneralConfig>,
    selected_device: State<'_, SelectedDevice>,
) -> Result<String, String> {
    let device = selected_device.lock().unwrap().clone();
    let config = general_config.lock().unwrap().clone();

    let save_path = storage::data_path().join(config.media_storage_dir);
    let file_path = save_path.join("preview.png");

    if path_exists(&file_path) {
        std::fs::remove_file(&file_path).unwrap();
    }

    if !path_exists(&save_path) {
        std::fs::create_dir_all(&save_path).unwrap();
    }

    // let preview = file_path.clone().to_str().unwrap().to_owned();

    let img_data = CameraController::
            take_snapshot(
                crate::CameraSnapshotOptions {
                    delay: config.preferences.webcam_delay,
                    save_path: file_path,
                    compress: false,
                    selected_device: device.human_name() }
            ).await.map_err(|err| {
                 eprint!("CameraController Error: {:?}", err);
                 err.to_string()
            })?;
    // tauri::async_runtime::spawn(async move {
    // });

    Ok(img_data)
}

#[tauri::command]
pub fn login(
    auth_config: State<'_, AuthConfig>,
    window: Window,
    payload: Auth,
) -> Result<(), String> {
    println!("Login: {:?}", &payload);
    let handle = window.app_handle();
    windows::close_login(&handle);

    *auth_config.lock().unwrap() = Some(payload.clone());
    storage::save_to_path(&payload, storage::auth_path::<Auth>()).map_err(|err| err.to_string())?;

    windows::show_tracker(&handle);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShowWindowPayload {
    pub name: String,
}

#[tauri::command]
pub fn show_window(app: AppHandle, name: String) -> Result<(), String> {
    let window = AppWindow::from_label(&name);
    window.show(&app).ok();
    Ok(())
}

#[tauri::command]
pub fn hide_window(window: Window, _name: String) {
    // let window = AppWindow::from_label(&name);
    window.hide().ok();
}

#[tauri::command]
pub fn minimize_window(window: Window, name: String) -> Result<(), String> {
    if let Some(window) = window.app_handle().get_window(&name) {
        window
            .minimize()
            .map_err(|err| format!("Error minimizing window: {err}"))?;
        return Ok(());
    }

    Ok(())
}

#[tauri::command]
pub fn list_camera_devices() -> Result<Vec<String>, String> {
    let backend = native_api_backend().unwrap();
    let devices = nokhwa::query(backend).map_err(|err| {
        println!("nokhwa::query(backend) error: {:?}", err);
        format!("Error listing camera devices: {:?}", err)
    })?;
    println!("[list_camera_devices] {:?}", devices);
    Ok(devices.iter().map(|camera| camera.human_name()).collect())
}

#[tauri::command]
pub fn select_camera_device(selected_device: State<'_, SelectedDevice>, name: String) {
    let backend = native_api_backend().unwrap();
    match nokhwa::query(backend) {
        Ok(devices) => {
            let mut devices = devices;
            devices.retain(|camera| camera.human_name() == name.as_str());

            println!("[camera_devices] {:?}", devices);

            #[allow(clippy::get_first)]
            if let Some(camera) = devices.get(0) {
                println!("Selected device: {:?}", camera);
                *selected_device.lock().unwrap() = camera.to_owned();
            }
        }
        Err(err) => {
            println!("nokhwa::query(backend) error: {:?}", err);
        }
    };
}

#[tauri::command]
pub fn get_selected_camera_device(
    selected_device: State<'_, SelectedDevice>,
) -> Result<CameraInfo, String> {
    Ok(selected_device.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_track_history(
    time_tracker: State<'_, TimeTrackerMap>,
) -> Result<TrackHistory, String> {
    Ok(time_tracker.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_time_tracked_today(
    time_tracker: State<'_, TimeTrackerMap>,
) -> Result<u64, String> {
    Ok(time_tracker.lock().unwrap().get_track_for_today())
}

#[tauri::command]
pub fn quit_app(
) {
    std::process::exit(0);
}
