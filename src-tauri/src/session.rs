use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

// use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::broadcast;
use xcap::Monitor;

use crate::{gen_rand_string, get_current_datetime, get_storage_path, AppState, Shutdown};

pub type SessionChannel = tokio::sync::broadcast::Sender<()>;
pub type SessionState = Arc<Mutex<Session>>;

pub type DateTimeTz = String;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown: Arc<Shutdown>,
}

#[derive(Debug, Clone)]
pub struct TimeCapsule {
    pub id: String,
    pub session_id: String,
    pub mouse_clicks: Arc<RwLock<Vec<DateTimeTz>>>,
    pub keystrokes: Arc<RwLock<Vec<DateTimeTz>>>,
    pub media: Arc<RwLock<Vec<String>>>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageTimeCapsule {
    pub id: String,
    pub session_id: String,
    pub mouse_clicks: Vec<DateTimeTz>,
    pub keystrokes: Vec<DateTimeTz>,
    pub media: Vec<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

impl Session {
    pub async fn start(&self, app: AppHandle) -> crate::Result<()> {
        // let record_channel = app.state::<RecordChannel>().clone();

        let mut shutdown = Shutdown::new(self.notify_shutdown.subscribe());
        let mut is_shutdown = false;
        while !shutdown.is_shutdown() {
            let mut time_capsule = TimeCapsule {
                id: gen_rand_string(16),
                session_id: self.id.clone(),
                mouse_clicks: Arc::new(RwLock::new(vec![])),
                keystrokes: Arc::new(RwLock::new(vec![])),
                media: Arc::new(RwLock::new(vec![])),
                started_at: get_current_datetime().to_rfc2822(),
                ended_at: None,
            };

            let capsule_id = time_capsule.id.clone();
            println!(
                "Start time capsule: {}, is_shutdown: {is_shutdown}",
                capsule_id
            );

            tokio::select! {
                res = time_capsule
                    .record(app.clone(), Shutdown::new(self.notify_shutdown.subscribe())) => {
                        match res {
                                Ok(signal) => {
                                    is_shutdown = signal;
                                    println!("Timecapsule Finished. Shutdown: {signal}");
                                },
                            Err(err) => {
                                // log error to server
                                // save to local error logs to be sent later incase of network issues
                                println!(
                                    "Error: Timecapsule {} crashed, details  {:?}",
                                    capsule_id, err
                                );
                            }
                            };
                    },
                _ = shutdown.recv() => {
                    println!("Shutdown signal: {}, session done: {}", shutdown.is_shutdown(), self.shutdown.is_shutdown());

                }
            }

            let data_path = tauri::api::path::app_data_dir(&app.config()).unwrap_or_default();
            let storage_path = data_path.join("capsules");

            tokio::spawn(async move {
                if let Err(err) = save_capsule(time_capsule, storage_path).await {
                    println!("Couldn't save time capsule: {:?}", err);
                    // todo: log error to server and save to local error log
                }
            });

            // self.time_capsules.push(time_capsule);
            println!("Added time capsule");
        }

        println!("Session shutdown");

        Ok(())
    }

    pub fn is_stopped(&self) -> bool {
        self.shutdown.is_shutdown()
    }
}

const SESSION_TIME: u16 = 30;
const MIN_MEDIA_CAPTURE_TIME: u16 = 3;
const MEDIA_CAPTURE_LAG: u16 = 20;

impl TimeCapsule {
    /// Record all activities within the time capsule
    ///
    /// Returns bool true if recording ends normally
    /// Returns bool false if shutdown signal was received during recording
    pub async fn record(
        &mut self,
        app_handle: AppHandle,
        shutdown: Shutdown,
    ) -> crate::Result<bool> {
        let state = app_handle.state::<AppState>();
        let mut mouse_click_rx = state.mouseclick_rx.as_ref().unwrap().subscribe();
        let mut keystroke_rx = state.keystroke_rx.as_ref().unwrap().subscribe();

        let mouseclicks = Arc::clone(&self.mouse_clicks);
        let keystrokes = Arc::clone(&self.keystrokes);

        let (notify_end, _) = broadcast::channel::<()>(2);
        let mut mouseclick_shutdown = Shutdown::new(notify_end.subscribe());
        let mut keystroke_shutdown = Shutdown::new(notify_end.subscribe());

        let listen_for_mouse_clicks = async move {
            while !mouseclick_shutdown.is_shutdown() {
                tokio::select! {
                    _ = mouseclick_shutdown.recv() => {
                         println!("mouseclick listener is shutting down");
                         break;
                    },
                    resp = mouse_click_rx.recv() => {
                        match resp {
                            Ok(dt) => {
                                mouseclicks.write().unwrap().push(dt.to_rfc2822());
                                // mouseclicks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                println!(
                                    "Mouse click: {} at: {}",
                                    // mouseclicks.load(std::sync::atomic::Ordering::SeqCst),
                                    mouseclicks.read().unwrap().len(),
                                    dt.to_rfc2822()
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
                         println!("keystroke task is shutting down");
                         break;
                    },
                    resp = keystroke_rx.recv() => {
                        match resp {
                            Ok(dt) => {
                                keystrokes.write().unwrap().push(dt.to_rfc2822());
                                // keystrokes.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                println!(
                                    "Key stroke: {} at: {}",
                                     keystrokes.read().unwrap().len(),
                                    // keystrokes.load(std::sync::atomic::Ordering::SeqCst),
                                    dt.to_rfc2822()
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

        let storage_path = get_storage_path(&app_handle).unwrap();

        let max_delay_based_on_capture_lag = SESSION_TIME - MEDIA_CAPTURE_LAG;
        let delay = {
            let mut gen = thread_rng();
            gen.gen_range(MIN_MEDIA_CAPTURE_TIME..=max_delay_based_on_capture_lag)
        };

        let screenshots = Arc::clone(&self.media);
        let media_capture_task = tokio::spawn(async move {
            println!("Media capture scheduled to run at: {delay} seconds");
            tokio::time::sleep(Duration::from_secs(delay as u64)).await;

            let monitors = Monitor::all().unwrap();
            // let storage_path = storage_path.unwrap();
            for monitor in monitors {
                // println!("Monitor: {:?}", monitor.name());
                let image = monitor.capture_image().unwrap();
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let img_path = storage_path.clone().join(format!(
                    "{}-{}.png",
                    timestamp.as_nanos(),
                    monitor.name(),
                ));

                let file = tokio::fs::File::create(img_path.clone()).await;
                // println!("PreSave: {:?}, Exists: {}", &img_path, file.is_ok());

                if file.is_ok() {
                    // println!("Save: {:?}", &img_path);
                    image.save(&img_path).unwrap();
                    screenshots
                        .write()
                        .unwrap()
                        .push(img_path.to_str().unwrap().to_owned());
                }
            }

            // println!(
            //     "Screencapture Done: {:?}, media: {:?}",
            //     start.elapsed(),
            //     screenshots.read().unwrap()
            // );

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

            tokio::time::sleep(Duration::from_secs(
                (SESSION_TIME - max_delay_based_on_capture_lag) as u64,
            ))
            .await;
            // }
        });

        let timeout = tokio::spawn(tokio::time::sleep(Duration::from_secs(SESSION_TIME as u64)));

        let mouseclick_task = tokio::spawn(listen_for_mouse_clicks);
        let keystroke_task = tokio::spawn(listen_for_keystrokes);

        let mut shutdown = shutdown;
        let mut shutdown_signal_received = false;
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
            _ = timeout => println!("Sessioon Timeout"),
            _ = shutdown.recv() => {
                shutdown_signal_received = true;
                println!("Shutdown signal received");
                drop(notify_end.clone());
            }
        }

        // send drop signals
        drop(notify_end);

        println!("Session ended {}", self.id);
        // println!(
        //     "Mouse clicks: {}, keys strokes: {}",
        //     self.mouse_clicks.load(std::sync::atomic::Ordering::SeqCst),
        //     self.keystrokes.load(std::sync::atomic::Ordering::SeqCst),
        // );

        self.ended_at = Some(get_current_datetime().to_rfc2822());
        Ok(shutdown_signal_received)
    }
}

async fn save_capsule(time_capsule: TimeCapsule, storage_path: PathBuf) -> crate::Result<()> {
    let TimeCapsule {
        id,
        media,
        ended_at,
        started_at,
        session_id,
        keystrokes,
        mouse_clicks,
    } = time_capsule;

    let value = StorageTimeCapsule {
        id,
        ended_at,
        started_at,
        session_id,
        media: media.read().unwrap().clone(),
        mouse_clicks: mouse_clicks.read().unwrap().clone(),
        keystrokes: keystrokes.read().unwrap().clone(),
    };

    let capsule = serde_json::to_string(&value)?;

    // let data_path = tauri::api::path::app_data_dir(&app_handle.config()).unwrap_or_default();
    // let storage_path = data_path.join("timecapsules");
    let path = storage_path.to_str().unwrap();

    let exists = std::fs::metadata(path).map_or(false, |_| true);
    println!("Exits: {:?}", exists);
    if !exists {
        match std::fs::create_dir(&storage_path) {
            Ok(()) => {}
            Err(err) => println!("Error creating storage folder: {:?}", err),
        }
    }

    let file = std::fs::File::create(storage_path.join(format!("{}.json", value.id)))?;
    if let Err(err) = serde_json::to_writer(&file, &capsule) {
        println!("Error saving session: {} :: {:?}", value.id, err);
    }

    println!("Session {} saved", value.id);

    Ok(())
}
