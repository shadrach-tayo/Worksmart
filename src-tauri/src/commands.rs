#![allow(unused_imports)]

use chrono::Utc;
use core_graphics::access::ScreenCaptureAccess;

use gst::prelude::*;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::{backends::capture::*, Camera};
use nokhwa::{native_api_backend, pixel_format};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
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

use crate::data_path;
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

#[tauri::command]
pub async fn start_session(
    window: Window,
    session_rx: State<'_, SessionChannel>,
    session: State<'_, SessionState>,
    general_config: State<'_, GeneralConfig>,
) -> Result<(), ()> {
    let app_handle = window.app_handle();

    println!("{:?}", general_config.lock().unwrap());

    *session.lock().unwrap() = Session {
        id: gen_rand_string(16),
        started_at: Some(get_current_datetime().to_rfc2822()),
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
    Ok(())
}

#[tauri::command]
pub async fn stop_session(window: Window, session_rx: State<'_, SessionChannel>) -> Result<(), ()> {
    let app_handle = window.app_handle();

    if app_handle
        .state::<SessionState>()
        .lock()
        .unwrap()
        .started_at
        .is_some()
    {
        session_rx.send(()).unwrap();
        std::thread::yield_now();
        app_handle.state::<SessionState>().lock().unwrap().ended_at =
            Some(get_current_datetime().to_rfc2822());
    }

    let state = app_handle.state::<SessionState>().lock().unwrap().clone();

    println!(
        "Session gracefully from: {:?} to: {:?}",
        state.started_at, state.ended_at
    );

    println!("I'm out");
    Ok(())
}

#[tauri::command]
pub async fn get_session(session: State<'_, SessionState>) -> Result<Session, ()> {
    Ok(session.lock().unwrap().clone())
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
pub fn update_config(general_config: State<'_, GeneralConfig>) {
    // ideally take general config struct from client and save first
    // then assign it as new general_config: config_state.lock().unwrap() = configuration

    general_config
        .lock()
        .unwrap()
        .preferences
        .time_gap_duration_in_seconds = 1200;

    configuration::save(&general_config.lock().unwrap().clone());

    println!(
        "Config Updated: {:?}",
        general_config.lock().unwrap().clone()
    );
}

#[tauri::command]
pub fn webcam_capture(general_config: State<'_, GeneralConfig>) -> Result<(), String> {
    let config = general_config.lock().unwrap().clone();

    let is_granted = nokhwa::nokhwa_check();
    if !is_granted {
        println!("Permission not granted: {is_granted}");
        return Err("Permission required!".into());
    }

    let backend = native_api_backend().unwrap();
    let devices = nokhwa::query(backend).unwrap();
    println!("There are {} available cameras.", devices.len());
    for device in devices {
        println!("{device}");
    }

    // first camera in system
    let index = CameraIndex::Index(0);
    // request the absolute highest resolution CameraFormat that can be decoded to RGB.
    let resolution = Resolution::new(1920, 1080);

    let f_format = FrameFormat::NV12;
    let fps = 30;
    let camera_format = CameraFormat::new(resolution, f_format, fps);

    let requested = RequestedFormat::new::<pixel_format::RgbFormat>(RequestedFormatType::Closest(
        camera_format,
    ));

    println!("Camera {:?} Request {:?}", &index, &requested);
    // make the camera
    let mut camera = Camera::new(index, requested).unwrap();
    println!(
        "Camera created : {:?}, {:?}",
        camera.resolution(),
        camera.camera_format()
    );
    camera.open_stream().unwrap();

    // get a frame
    println!(
        "Frame format: {:?}, camera_format: {:?}",
        camera.frame_format(),
        camera.camera_format()
    );
    let frame = camera.frame().unwrap();
    camera.stop_stream().unwrap();
    println!("Captured Single Frame of {}", frame.buffer().len());
    // decode into an ImageBuffer
    let decoded = frame.decode_image::<pixel_format::RgbFormat>().unwrap();
    println!("Decoded Frame of {}", decoded.len());
    // std::fs::File::create(&path).expect("Cannot not save webcam image");
    let path = data_path().join(config.media_storage_dir.clone().join("webcam.jpeg"));
    if let Err(err) = decoded.save(path) {
        println!("Error saving webcam image {:?}", err);
    }
    // match camera.frame() {
    //     Ok(frame) => {
    //     }
    //     Err(err) => {
    //         println!("Failed to get frame: {:?}", err);
    //     }
    // }

    Ok(())
}
