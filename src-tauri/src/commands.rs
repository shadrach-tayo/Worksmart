#![allow(unused_imports)]

use chrono::Utc;
use core_graphics::access::ScreenCaptureAccess;

use gst::prelude::*;
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
