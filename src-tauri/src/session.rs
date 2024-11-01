use std::{
    path::PathBuf, sync::{self, atomic::AtomicBool, Arc, Mutex, RwLock}, time::Duration
};

use chrono::Utc;
// use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::broadcast;
use xcap::Monitor;

use crate::{gen_rand_string, get_current_datetime, get_focused_window, storage, AppState, CameraController, CameraSnapshotOptions, GeneralConfig, SelectedDevice, Shutdown, TimeTrackerMap};

pub type SessionChannel = tokio::sync::broadcast::Sender<()>;
pub type SessionState = Arc<Mutex<Session>>;

pub type DateTimeTz = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEntry {
    name: String,
    title: String,
    time: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub is_running: bool,
    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown: Arc<Shutdown>,
}

#[derive(Debug, Clone)]
pub struct TimeCapsule {
    pub id: String,
    pub session_id: String,
    pub mouse_clicks: Arc<RwLock<Vec<DateTimeTz>>>,
    pub keystrokes: Arc<RwLock<Vec<DateTimeTz>>>,
    pub windows: Arc<RwLock<Vec<WindowEntry>>>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub storage_path: PathBuf,
    exited: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageTimeCapsule {
    pub id: String,
    pub session_id: String,
    pub mouse_clicks: Vec<DateTimeTz>,
    pub keystrokes: Vec<DateTimeTz>,
    pub windows: Vec<WindowEntry>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

impl Session {
    pub async fn start(&self, app: AppHandle) -> crate::Result<()> {
        // let record_channel = app.state::<RecordChannel>().clone();

        let mut shutdown = Shutdown::new(self.notify_shutdown.subscribe());
        let mut is_shutdown = false;

        while !is_shutdown {
            let id = gen_rand_string(16);
            let dir =  app.state::<GeneralConfig>()
                .lock()
                .unwrap()
                .capsule_storage_dir
                .clone();

            let storage_path = storage::data_path().join(dir).join(id.clone());
            std::fs::create_dir_all(&storage_path).expect("Can't create capsule directory");

            let mut time_capsule = TimeCapsule {
                id,
                storage_path,
                session_id: self.id.clone(),
                mouse_clicks: Arc::new(RwLock::new(vec![])),
                keystrokes: Arc::new(RwLock::new(vec![])),
                windows: Arc::new(RwLock::new(vec![])),
                started_at: get_current_datetime().to_rfc2822(),
                ended_at: None,
                exited: Arc::new(AtomicBool::new(false)),

            };

            let capsule_id = time_capsule.id.clone();
            println!(
                "Start time capsule: {}, is_shutdown: {is_shutdown}",
                capsule_id
            );

            let start_ts = Utc::now().timestamp() as u64;

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
                    is_shutdown = true;
                    println!("Shutdown signal: {}, session done: {}", shutdown.is_shutdown(), self.shutdown.is_shutdown());

                }
            }

            time_capsule.exit();

            let handle = app.clone();
            // let start_ts = DateTime::parse_from_str(&time_capsule.started_at, "").unwrap().timestamp() as u64;
            let end_ts = Utc::now().timestamp() as u64;
 // time_capsule.ended_at.clone().map_or(Utc::now().timestamp() as u64, |end| DateTime::parse_from_str(&end, "").unwrap().timestamp() as u64);

            tokio::spawn(async move {
                if let Err(err) = save_capsule(time_capsule).await {
                    println!("Couldn't save time capsule: {:?}", err);
                    // todo: log error to server and save to local error log
                }

                // let start_dt = DateTime::parse_from_str(&started_at, "").unwrap().timestamp() as u64;
                // let end_dt = DateTime::parse_from_str(&started_at, "").unwrap().timestamp() as u64; //Utc::now().timestamp() as u64;
                let diff = end_ts - start_ts;
                dbg!(diff);
                handle.state::<TimeTrackerMap>().lock().unwrap().increment_track_for_today(diff);
                handle.state::<TimeTrackerMap>().lock().unwrap().save();

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
        let time_gap_in_secs = app_handle
            .state::<GeneralConfig>()
            .lock()
            .unwrap()
            .preferences
            .time_gap_duration_in_seconds;

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
                                // println!("MouseClick Event: {:?}", &dt.to_rfc3339());
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
        tokio::spawn(listen_for_mouse_clicks);


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
                                // println!("keystroke Event: {:?}", &dt);
                                keystrokes.write().unwrap().push(dt.to_rfc2822());
                            }
                            Err(err) => {
                                println!("keystroke receiver error: {:?}", err);
                            }
                        }
                    }
                }
            }

            // drop(keystroke_rx);
        };
        tokio::spawn(listen_for_keystrokes);

        let exited = self.exited.clone();
        // let mut active_windows = Arc::new(Mutex::new(vec![]));
        let active_windows = Arc::clone(&self.windows);
        let log_delay_in_seconds = time_gap_in_secs / 10;
        let active_window_logger = async move {
            // initial 10 secs delay before tracking active window
            tokio::time::sleep(Duration::from_secs(10)).await;

            let mut active_window = get_focused_window();

            if active_window.is_some() {
                let win = active_window.clone().unwrap();
                active_windows
                    .write()
                    .unwrap()
                    .push(WindowEntry { name: win.app_name, title: win.title, time: get_current_datetime().to_rfc3339() });
            }

            while !exited.load(sync::atomic::Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(log_delay_in_seconds)).await;
                if let Some(window) = get_focused_window() {
                    if active_window.is_some() && active_window.clone().unwrap().app_name != window.app_name.as_str() {
                        active_window = Some(window.clone());
                        active_windows
                            .write()
                            .unwrap()
                            .push(WindowEntry { name: window.app_name, title: window.title, time: get_current_datetime().to_rfc3339() });
                    }
                }

            }
        };
        tokio::spawn(active_window_logger);

        let storage_path = Arc::new(self.storage_path.clone());


        let max_delay_based_on_capture_lag = time_gap_in_secs - MEDIA_CAPTURE_LAG;
        let min_capture_start_time = time_gap_in_secs / 10;
        let delay = {
            let mut gen = thread_rng();
            gen.gen_range(min_capture_start_time..=max_delay_based_on_capture_lag)
        };


        let media_storage_path = Arc::clone(&storage_path);
        let capsule_exited = self.exited.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(delay as u64)).await;

            if capsule_exited.load(sync::atomic::Ordering::SeqCst) {
                return;
            }

            let monitors = Monitor::all().unwrap();
            let focused_window = match get_focused_window() {
                Some(w) => w.app_name,
                None => "".to_owned()
            };

            for monitor in monitors {
                let image = monitor.capture_image().unwrap();

                let window_name = if focused_window.is_empty() {
                    monitor.name()
                } else {
                    &focused_window
                };

                let img_path = media_storage_path.clone().join(format!(
                    "screenshot_{}_{}.png",
                    window_name,
                    get_current_datetime().to_rfc3339(),
                ));

                let file = tokio::fs::File::create(img_path.clone()).await;

                if file.is_ok() {
                    image.save(&img_path).unwrap();
                } else {
                    // save to error log and stream to server later
                    println!("Error saving screenshot to dir: {:?}", img_path);
                }
            }
        });

        let selected_device = app_handle.state::<SelectedDevice>().lock().unwrap().clone().human_name();
        let webcam_storage_path = Arc::clone(&storage_path);

        let exited = self.exited.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(delay as u64)).await;

            if exited.load(sync::atomic::Ordering::SeqCst) {
                return;
            }

            if let Err(err) = CameraController
                ::take_snapshot(
                    CameraSnapshotOptions {
                        save_path: webcam_storage_path.to_path_buf(),
                        selected_device
                    }
                ).await
            {
                eprintln!("Webcam snapshot error: {err}");
            }
        });

        let timeout = tokio::spawn(tokio::time::sleep(Duration::from_secs(time_gap_in_secs)));

        let mut shutdown = shutdown;
        let mut shutdown_signal_received = false;

        tokio::select! {
            _ = timeout => println!("Sessioon Timeout"),
            _ = shutdown.recv() => {
                shutdown_signal_received = true;
                println!("Shutdown signal received");
                drop(notify_end.clone());
            }
        }

        // send drop signals
        drop(notify_end);

        println!("Time capsule ended {}", self.id);
        self.ended_at = Some(get_current_datetime().to_rfc2822());
        Ok(shutdown_signal_received)
    }

    pub fn exit(&mut self) {
        self.exited.store(true ,sync::atomic::Ordering::SeqCst);
        println!("Exited: {}", self.exited.load(sync::atomic::Ordering::SeqCst));
    }
}

async fn save_capsule(time_capsule: TimeCapsule) -> crate::Result<()> {

    let TimeCapsule {
        id,
        windows,
        ended_at,
        started_at,
        session_id,
        keystrokes,
        mouse_clicks,
        storage_path,
        exited: _,
    } = time_capsule;


    let value = StorageTimeCapsule {
        id,
        ended_at,
        started_at,
        session_id,
        windows: windows.read().unwrap().clone(),
        mouse_clicks: mouse_clicks.read().unwrap().clone(),
        keystrokes: keystrokes.read().unwrap().clone(),
    };

    dbg!(&value);

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

    storage::save_to_data_path(&value, storage_path.join("metadata.json"));

    Ok(())
}
