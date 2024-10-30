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
use std::io::{Cursor, Read};
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

// use crate::encoder::uyvy422_frame;
use crate::{storage, windows, Auth, AuthConfig, CameraController, Configuration, SelectedDevice};

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
    general_config: State<'_, GeneralConfig>,
) -> Result<Option<SessionDetail>, ()> {
    let sesh = session.lock().unwrap().clone();
    if sesh.is_running {
        return Ok(None);
    }

    let app_handle = window.app_handle();

    println!("{:?}", general_config.lock().unwrap());

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

    let active_session = session.lock().unwrap().clone();

    tokio::spawn(async move {
        if let Err(err) = active_session.start(app_handle).await {
            println!("Session Error: {:?}", err);
        }
        println!("Close session#start thread");
    });

    // drop(session);

    println!("Lock dropped");
    Ok(Some(SessionDetail {
        id,
        started_at,
        ended_at: None,
    }))
}

#[tauri::command]
pub async fn stop_session(
    window: Window,
    session: State<'_, SessionState>,
    session_rx: State<'_, SessionChannel>,
) -> Result<(), ()> {
    if !session.lock().unwrap().is_running {
        return Ok(());
    }

    let app_handle = window.app_handle();

    session_rx.send(()).unwrap();
    // std::thread::yield_now();
    session.lock().unwrap().ended_at = Some(get_current_datetime().to_rfc3339());
    // session.lock().unwrap().ended_at = Some(get_current_datetime().to_rfc3339());
    session.lock().unwrap().is_running = false;

    let state = app_handle.state::<SessionState>().lock().unwrap().clone();

    println!(
        "Session gracefully from: {:?} to: {:?}",
        state.started_at, state.ended_at
    );

    println!("I'm out");
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

#[allow(clippy::default_constructed_unit_structs)]
#[tauri::command]
pub fn permissions_granted() -> bool {
    ScreenCaptureAccess::default().preflight() && scap::has_permission()
}

#[allow(clippy::default_constructed_unit_structs)]
#[tauri::command]
pub fn request_permissions() -> bool {
    ScreenCaptureAccess::default().request() && scap::request_permission()
}

#[tauri::command]
pub fn set_preferences(
    general_config: State<'_, GeneralConfig>,
    preferences: Configuration,
) -> Result<(), String> {
    *general_config.lock().unwrap() = preferences;

    storage::save(&general_config.lock().unwrap().clone());

    Ok(())
}

#[tauri::command]
pub fn get_preferences(general_config: State<'_, GeneralConfig>) -> Result<Configuration, String> {
    let config = general_config.lock().unwrap().clone();

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
pub fn webcam_capture(
    general_config: State<'_, GeneralConfig>,
    selected_device: State<'_, SelectedDevice>,
) -> Result<(), String> {
    let device = selected_device.lock().unwrap().clone();
    let config = general_config.lock().unwrap();
    let save_path = storage::data_path().join(config.media_storage_dir.clone());

    tauri::async_runtime::spawn(async move {
        if let Err(err) = CameraController::
            take_snapshot(
                crate::CameraSnapshotOptions {
                    save_path,
                    selected_device: device.human_name() }
            ).await {
                eprint!("CameraController Error: {:?}", err);
            }
    });

    Ok(())
}

#[tauri::command]
pub fn login(
    auth_config: State<'_, AuthConfig>,
    window: Window,
    payload: Auth,
) -> Result<(), String> {
    println!("Login: {:?}", &payload);
    let handle = window.app_handle();
    windows::close_login(&handle).map_err(|err| err.to_string())?;

    *auth_config.lock().unwrap() = Some(payload.clone());
    storage::save_to_path(&payload, storage::auth_path::<Auth>()).map_err(|err| err.to_string())?;

    windows::show_tracker(&handle).map_err(|err| err.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShowWindowPayload {
    pub name: String,
}

#[tauri::command]
pub fn show_window(window: Window, name: String) -> Result<(), String> {
    if let Some(window) = window.app_handle().get_window(&name) {
        window
            .show()
            .map_err(|err| format!("Error showing window: {err}"))?;
        return Ok(());
    }

    Ok(())
}

#[tauri::command]
pub fn hide_window(window: Window, name: String) -> Result<(), String> {
    if let Some(window) = window.app_handle().get_window(&name) {
        window
            .hide()
            .map_err(|err| format!("Error closing window: {err}"))?;
        return Ok(());
    }

    Ok(())
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
