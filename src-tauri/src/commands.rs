use core_graphics::access::ScreenCaptureAccess;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    ops::Mul,
    path::PathBuf,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, Window};
use tokio::{fs, sync::broadcast, time};
#[allow(unused_imports)]
use xcap::{Monitor, Window as XcapWindow};

use crate::{AppState, Result, Shutdown};

#[allow(clippy::default_constructed_unit_structs)]
fn has_permission() -> bool {
    ScreenCaptureAccess::default().preflight()
}

#[allow(clippy::default_constructed_unit_structs)]
fn request_permission() -> bool {
    ScreenCaptureAccess::default().request()
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Session {
    id: u64,
    mouse_clicks: usize,
    keystrokes: usize,
    media: Vec<String>,
}

const SESSION_TIME: u16 = 120;
const MEDIA_CAPTURE_LAG: u16 = 20;

#[tauri::command]
pub async fn capture_screen(window: Window) {
    // MacOs check permission
    if !has_permission() {
        println!("No Graphics access");
        let granted = request_permission();
        // prompt user to go grant access before they can use the app
        println!("Graphics access granted: {granted}");
    }

    let mut mouse_click_rx = window
        .app_handle()
        .state::<AppState>()
        .mouseclick_notifier
        .as_ref()
        .unwrap()
        .subscribe();
    let mut keystroke_rx = window
        .app_handle()
        .state::<AppState>()
        .key_notifier
        .as_ref()
        .unwrap()
        .subscribe();

    let session = Arc::new(Session {
        id: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .mul(1000),
        ..Default::default()
    });

    let mclicks = Arc::new(AtomicUsize::new(0));
    let mouseclicks = Arc::clone(&mclicks);
    let kclicks = Arc::new(AtomicUsize::new(0));
    let keystrokes = Arc::clone(&kclicks);

    let (notify_end, _) = broadcast::channel::<()>(1);
    // let shutdown = Shutdown::new(notify_end.subscribe());
    let mut mouseclick_shutdown = Shutdown::new(notify_end.subscribe());
    let mut keystroke_shutdown = Shutdown::new(notify_end.subscribe());

    let listen_for_mouse_clicks = async move {
        while !mouseclick_shutdown.is_shutdown() {
            tokio::select! {
                _ = mouseclick_shutdown.recv() => {
                     println!("mouseclick task is shutdown");
                     break;
                },
                resp = mouse_click_rx.recv() => {
                    match resp {
                        Ok(_) => {
                            mouseclicks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            println!(
                                "Mouse click: {}",
                                mouseclicks.load(std::sync::atomic::Ordering::SeqCst)
                            );
                        }
                        Err(err) => {
                            println!("mouse click Error: {:?}", err);
                        }
                    }
                }
            }
        }
    };

    let listen_for_keystrokes = async move {
        while !keystroke_shutdown.is_shutdown() {
            tokio::select! {
                _ = keystroke_shutdown.recv() => {
                     println!("mouseclick task is shutdown");
                     break;
                },
                resp = keystroke_rx.recv() => {
                    match resp {
                        Ok(_) => {
                            keystrokes.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            println!(
                                "Key stroke: {}",
                                keystrokes.load(std::sync::atomic::Ordering::SeqCst)
                            );
                        }
                        Err(err) => {
                            println!("keystroke receiver error: {:?}", err);
                        }
                    }
                }
            }
        }
        println!("keystroke is shutdown");
    };

    let app_handle = window.app_handle();
    let storage_path = get_storage_path(&app_handle).await;

    let session_assets = Arc::new(Mutex::new(vec![]));
    let media_session = Arc::clone(&session_assets);

    let max_delay_based_on_capture_lag = SESSION_TIME - MEDIA_CAPTURE_LAG;
    let delay = {
        let mut gen = thread_rng();
        gen.gen_range(20..=max_delay_based_on_capture_lag)
    };

    let media_capture_task = tokio::spawn(async move {
        println!("Media capture scheduled to run at: {delay} seconds");
        time::sleep(Duration::from_secs(delay as u64)).await;

        println!("Media capture Running");
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
                    image.save(&img_path).unwrap();
                    media_session
                        .lock()
                        .unwrap()
                        .push(img_path.to_str().unwrap().to_owned());
                }
            }

            println!("Screencapture Done: {:?}", start.elapsed());

            // let windows = XcapWindow::all().unwrap();

            // let start = Instant::now();
            // for window in windows {
            //     if window.is_minimized() {
            //         continue;
            //     }

            //     println!(
            //         "Window: {:?} {:?} {:?}",
            //         window.title(),
            //         (window.x(), window.y(), window.width(), window.height()),
            //         (window.is_minimized(), window.is_maximized())
            //     );

            //     let image = window.capture_image().unwrap();
            //     let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            //     let img_path = storage_path.clone().join(format!(
            //         "{}-{}.png",
            //         timestamp.as_nanos(),
            //         normalized(window.title()),
            //     ));

            //     let file = fs::File::create(img_path.clone()).await;
            //     println!("PreSave: {:?}, Exists: {}", &img_path, file.is_ok());
            //     image.save(&img_path).unwrap();
            //     media_session
            //         .lock()
            //         .unwrap()
            //         .push(img_path.to_str().unwrap().to_owned());
            // }
            // println!("Windows Capture Done: {:?}", start.elapsed());

            time::sleep(Duration::from_secs(
                (SESSION_TIME - max_delay_based_on_capture_lag) as u64,
            ))
            .await;
        }
    });

    let timeout = tokio::spawn(tokio::time::sleep(Duration::from_secs(SESSION_TIME as u64)));

    let mouseclick_task = tokio::spawn(listen_for_mouse_clicks);
    let keystroke_task = tokio::spawn(listen_for_keystrokes);

    tokio::select! {
        _ = media_capture_task => {
            println!("Media capture task exited!");
        },
        _ = mouseclick_task => {
            println!("Mouse click listener shutdown");
        },
        _ = keystroke_task => {
            println!("Keystroke listner shutdown");
        },
        _ = timeout => println!("60 secs Timeout")
    }

    // shutdown setssion
    drop(notify_end);

    println!("Session {}", session.id);
    println!(
        "Mouse clicks: {}, keys strokes: {}",
        mclicks.load(std::sync::atomic::Ordering::SeqCst),
        kclicks.load(std::sync::atomic::Ordering::SeqCst)
    );

    let mut session = Arc::try_unwrap(session).unwrap();
    session.media = session_assets.lock().unwrap().clone();
    drop(session_assets);

    // update session metrics
    session.keystrokes = kclicks.load(std::sync::atomic::Ordering::SeqCst);
    session.mouse_clicks = mclicks.load(std::sync::atomic::Ordering::SeqCst);

    let _ = save_session(session, &app_handle).await;
}

#[allow(dead_code)]
fn normalized(filename: &str) -> String {
    filename
        .replace("|", "")
        .replace("\\", "")
        .replace(":", "")
        .replace("/", "")
}

async fn save_session(session: Session, app_handle: &AppHandle) -> crate::Result<()> {
    let session_data = serde_json::to_string(&session)?;

    let data_path = tauri::api::path::app_data_dir(&app_handle.config()).unwrap_or_default();
    let storage_path = data_path.join("sessions");
    let path = storage_path.to_str().unwrap();

    println!("Storage path: {:?}", &storage_path);
    let exists = tokio::fs::try_exists(path).await?;
    if !exists {
        match fs::create_dir(&storage_path).await {
            Ok(()) => {}
            Err(err) => println!("Error creating storage folder: {:?}", err),
        }
    }

    let file = std::fs::File::create(storage_path.join(format!("{}", session.id)))?;
    if let Err(err) = serde_json::to_writer(&file, &session_data) {
        println!("Error saving session: {} :: {:?}", session.id, err);
    }

    println!("Session {} saved", session.id);

    Ok(())
}

async fn get_storage_path(app_handle: &AppHandle) -> Result<PathBuf> {
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
