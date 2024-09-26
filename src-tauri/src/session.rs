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

use crate::{
    data_path, gen_rand_string, get_current_datetime, save_to_data_path, AppState, GeneralConfig,
    Shutdown,
};

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
                                    println!("Timecapsule Finished. Shutdown signal received: {signal}");
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

            let storage_path = data_path().join(
                app.state::<GeneralConfig>()
                    .lock()
                    .unwrap()
                    .capsule_storage_dir
                    .clone(),
            );

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

// const SESSION_TIME: u16 = 30;
// const MIN_MEDIA_CAPTURE_TIME: u16 = 3;
const MEDIA_CAPTURE_LAG: u64 = 20;

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

        let storage_path = data_path().join(
            app_handle
                .state::<GeneralConfig>()
                .lock()
                .unwrap()
                .media_storage_dir
                .clone(),
        );

        let time_gap_in_secs = app_handle
            .state::<GeneralConfig>()
            .lock()
            .unwrap()
            .preferences
            .time_gap_duration_in_seconds;
        let max_delay_based_on_capture_lag = time_gap_in_secs - MEDIA_CAPTURE_LAG;
        let min_capture_start_time = time_gap_in_secs / 10;
        let delay = {
            let mut gen = thread_rng();
            gen.gen_range(min_capture_start_time..=max_delay_based_on_capture_lag)
        };

        let screenshots = Arc::clone(&self.media);
        let capsule_id = self.id.clone();
        let media_capture_task = tokio::spawn(async move {
            println!("Media capture scheduled to run at: {delay} seconds");
            tokio::time::sleep(Duration::from_secs(delay as u64)).await;

            let monitors = Monitor::all().unwrap();
            for monitor in monitors {
                let image = monitor.capture_image().unwrap();
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let img_path = storage_path.join(format!(
                    "{}-{}-{}.png",
                    capsule_id,
                    timestamp.as_nanos(),
                    monitor.name(),
                ));

                let file = tokio::fs::File::create(img_path.clone()).await;

                if file.is_ok() {
                    image.save(&img_path).unwrap();
                    screenshots
                        .write()
                        .unwrap()
                        .push(img_path.to_str().unwrap().to_owned());
                } else {
                    // save to error log and stream to server later
                    println!("Error saving screenshot to dir: {:?}", img_path);
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
                time_gap_in_secs - max_delay_based_on_capture_lag,
            ))
            .await;
            // }
        });

        let timeout = tokio::spawn(tokio::time::sleep(Duration::from_secs(time_gap_in_secs)));

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

    let path = storage_path.to_str().unwrap();

    let exists = std::fs::metadata(path).map_or(false, |_| true);
    if !exists {
        match std::fs::create_dir(&storage_path) {
            Ok(()) => {}
            Err(err) => {
                println!("Error creating storage folder: {:?}", err);
                panic!("Could not create storage directory")
            }
        }
    }

    save_to_data_path(&value, storage_path.join(format!("{}.json", value.id)));

    Ok(())
}
