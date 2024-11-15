// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#![allow(unused_imports)]

use std::{fs::Permissions, sync::{atomic::{self, AtomicBool}, Arc, Mutex}};
use chrono::{DateTime, Utc};
// use gst::prelude::*;

use scap::capturer::{Area, Capturer, Options, Point, Size};

use rdev::{listen, Event, EventType};
use tauri::{Manager, WindowEvent};

use worksmart::{
    autostart, commands, gen_rand_string, get_current_datetime, get_default_camera, get_storage_path, session::{SessionChannel, SessionController, SessionControllerState, SessionState}, state::{KeystrokeBroadCaster, MouseclickBroadCaster}, windows, AppState, Auth, AuthConfig, CameraController, Configuration, GeneralConfig, PermisssionsStatus, RecordChannel, SelectedDevice, Session, Shutdown, TimeTrackerMap, TrackHistory
};

pub fn create_device_query_listener(mouseclick_rx: MouseclickBroadCaster, keystroke_rx: KeystrokeBroadCaster) {
    tauri::async_runtime::spawn(async move {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::ButtonPress(rdev::Button::Left) => {
                    // println!("ButtonPress");
                    if let Err(err) =
                        mouseclick_rx
                        .send(get_current_datetime())
                    {
                        // print error log or send stat to server
                        eprintln!("Error broadcasting Mouse event: {:?}", err);
                    }
                }
                // EventType::Wheel { delta_x, delta_y } => {
                //     println!("Scroll captured: X: {delta_x} y: {delta_y}");
                //     if let Err(err) =
                //         keystroke_rx
                //         .send(get_current_datetime())
                //     {
                //         // print error log or send stat to server
                //         eprintln!("Error broadcasting Mouse wheel event: {:?}", err);
                //     }
                // }
                EventType::KeyPress(_) => {
                    // println!("KeyPress");
                    if let Err(err) = keystroke_rx
                        .send(get_current_datetime())
                    {
                        // print error log or send stat to server
                        eprintln!("Error broadcasting keystroke event: {:?}", err);
                    }
                }
                _ => (),
            };
        };

        if let Err(error) = listen(callback) {
            println!("Error listening for key/mouse events: {:?}", error)
        }
    });
}

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.to_string();
        eprintln!("{message}");
    }));

    // intialize tauri async runtime
    tauri::async_runtime::set(tokio::runtime::Handle::current());


    let (mouseclicks_broadcaster, _): (MouseclickBroadCaster, _) =
        tokio::sync::broadcast::channel::<DateTime<Utc>>(1);
    let (keystrokes_broadcaster, _): (KeystrokeBroadCaster, _) =
        tokio::sync::broadcast::channel::<DateTime<Utc>>(1);

    // attach mouse and click broadcaster/subscriber to app state
    // only call when work is in session and close when session has ended
    create_device_query_listener(mouseclicks_broadcaster.clone(), keystrokes_broadcaster.clone());

    // init gst
    // gst::init().unwrap();
    // let gst_registry = gst::Registry::get();
    // gst_registry.scan_path(std::env::current_exe().unwrap().parent().unwrap());

    #[cfg(target_os = "macos")]
    nokhwa::nokhwa_initialize(|granted| {
        println!("Camera permission granted: {granted}");
    });

    #[allow(unused_variables)]
    #[allow(unused_mut)]
    let (record_tx, mut record_rx): (RecordChannel, _) = tauri::async_runtime::channel(100);

    let (session_tx, _): (SessionChannel, _) = tokio::sync::broadcast::channel(1);

    let session: SessionState = Arc::new(Mutex::new(Session {
        id: gen_rand_string(16),
        started_at: None,
        ended_at: None,
        is_running: false,
        notify_shutdown: session_tx.clone(),
        shutdown: Arc::new(Shutdown::new(session_tx.subscribe())),
    }));

    let general_config: GeneralConfig = Arc::new(Mutex::new(Configuration::default()));
    println!("Config: {:?}", general_config.lock().unwrap().clone());

    let auth_config = {
        let auth = Auth::default();

        if auth.name.is_empty() {
            None
        } else {
            Some(auth)
        }
    };

    let auth_config: AuthConfig = Arc::new(Mutex::new(auth_config));

    let time_tracker: TimeTrackerMap = Arc::new(Mutex::new(TrackHistory::default()));

    let session_controller: SessionControllerState = Arc::new(Mutex::new(SessionController::default()));

    let selected_device: SelectedDevice = Arc::new(Mutex::new(get_default_camera().unwrap()));

    let app = tauri::Builder::default()
        .manage(AppState {
            mouseclick_rx: Some(mouseclicks_broadcaster),
            keystroke_rx: Some(keystrokes_broadcaster),
        })
        .manage(record_tx)
        .manage(session_tx)
        .manage(session)
        .manage(general_config)
        .manage(auth_config)
        .manage(time_tracker)
        .manage(selected_device)
        .manage(session_controller)
        .invoke_handler(tauri::generate_handler![
            commands::start_session,
            commands::stop_session,
            commands::get_session,
            commands::record_screen,
            commands::set_preferences,
            commands::get_preferences,
            commands::webcam_capture,
            commands::login,
            commands::get_auth,
            commands::show_window,
            commands::hide_window,
            commands::minimize_window,
            commands::list_camera_devices,
            commands::select_camera_device,
            commands::get_track_history,
            commands::get_time_tracked_today,
            commands::request_camera_permissions,
            commands::request_accessibility_permissions,
            commands::request_screen_capture_permissions,
            commands::on_permissions_granted,
            commands::get_permission_status,
            commands::quit_app,
        ])
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                println!("close request!");
                // if user is in session prevent close or end session first
                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        .setup(|app| {
            let permissions = PermisssionsStatus::get_status();
            if permissions.required_granted() {
                if app.app_handle().state::<AuthConfig>().lock().unwrap().is_none() {
                    windows::show_login(&app.app_handle());
                } else {
                    windows::show_tracker(&app.app_handle());
                }
            } else {
                windows::show_permission(&app.app_handle());
            }

            // purge stale keys
            app.state::<TimeTrackerMap>().lock().unwrap().clean_up();


            // tauri auto update
            let shared_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                match tauri::updater::builder(shared_handle).check().await {
                    Ok(update) => {
                        println!("Worksmart Update: {}", update.is_update_available());
                        if update.is_update_available() {
                            update.download_and_install().await.unwrap();
                        }
                    }
                    Err(e) => {
                        println!("worksmart Update failed to get update: {}", e);
                    }
                }
            });

            // autostart by default on debug mode (debug build)
            let is_debug_mode = cfg!(debug_assertions);

            // Enable app auto launch
            let autostart = autostart::update(!is_debug_mode);
            if autostart.is_ok() {
                println!(
                    "Auto start {}",
                    if !is_debug_mode {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }


            // todo: sign in on app launch based on user preference
            // todo: start tracking on app launch based on user preference

            // let media_data_clone = media_data.clone();
            // let output_path = get_storage_path(&app.app_handle()).unwrap();

            // let can_record: Arc<atomic::AtomicPtr<bool>> = Arc::new(atomic::AtomicPtr::new(Box::into_raw(Box::new(false))));
            // let should_capture = can_record.clone();

            // std::thread::spawn(move || {

            //     let media_data = Arc::new(Mutex::new(Vec::<u8>::new()));
            //     let mut running_pipeline: Option<gst::Pipeline> = None;

            //     // Create Recorder
            //     let targets = scap::get_targets();
            //     println!("🎯 Targets: {:?}", targets);

            //     // Create Options
            //     let options = Options {
            //         fps: 30,
            //         targets,
            //         show_cursor: true,
            //         show_highlight: true,
            //         excluded_targets: None,
            //         output_type: scap::frame::FrameType::BGRAFrame,
            //         output_resolution: scap::capturer::Resolution::_720p,
            //         source_rect: Some(Area {
            //             origin: Point { x: 0.0, y: 0.0 },
            //             size: Size {
            //                 width: 1280.0,
            //                 height: 720.0,
            //             },
            //         }),
            //         ..Default::default()
            //     };

            //     let recorder =  Arc::new(atomic::AtomicPtr::new(Box::into_raw(Box::new(
            //         Capturer::new(options.clone()),
            //     ))));


            //     loop {
            //         let Some(command) = record_rx.blocking_recv() else { continue };

            //         match command {
            //             recorder::RecordCommand::Start => {

            //                 let recording_start_time = std::time::Instant::now();
            //                 should_capture.store(Box::into_raw(Box::new(true)), atomic::Ordering::Relaxed);

            //                 let media_data_clone = media_data.clone();

            //                 let mut pipeline_description = Vec::<String>::new();
            //                 let mut input_callbacks: Vec<(String, gst_app::AppSrcCallbacks)> = Vec::new();

            //                 pipeline_description.push(format!(
            //                     "appsrc name=input ! rawvideoparse width={width} height={height} format=8 ! videoconvert ! x264enc ! mp4mux ! appsink name=output",
            //                         width = options.source_rect.as_ref().unwrap().size.width,
            //                         height = options.source_rect.as_ref().unwrap().size.height,
            //                 ));

            //                 // Start Capture
            //                 {
            //                     unsafe { &mut *recorder.load(atomic::Ordering::Acquire) }.start_capture();
            //                     println!("Screen recording started!");
            //                 }

            //                 {
            //                     let video_input_callback = gst_app::AppSrcCallbacks::builder()
            //                         .need_data({
            //                             let can_record = should_capture.clone();
            //                             let capturer = recorder.clone();

            //                             move |source, _| {
            //                                 println!(
            //                                     "video_input_callback called! should capture: {:?}",
            //                                     can_record.load(atomic::Ordering::Relaxed)
            //                                 );
            //                                 let bgra_buffer = loop {
            //                                     if ! unsafe{*can_record.load(atomic::Ordering::Relaxed)} {
            //                                         println!("Screen recording stopped!");
            //                                         source.end_of_stream().unwrap();
            //                                         return;
            //                                     }

            //                                     let frame =
            //                                         match unsafe { &mut *capturer.load(atomic::Ordering::Acquire) }
            //                                             .get_next_frame()
            //                                         {
            //                                             Ok(frame) => frame,
            //                                             Err(err) => {
            //                                                 println!("Received error in frame getter: {:?}", err);
            //                                                 continue;
            //                                             }
            //                                         };
            //                                     // .expect("Error capturing next frame");

            //                                     let scap::frame::Frame::BGRA(pixel) = frame else {
            //                                         eprintln!("Received frame is not BGRA, skipping!");
            //                                         continue;
            //                                     };

            //                                     break pixel;
            //                                 };

            //                                 let pts = std::time::Instant::now() - recording_start_time;
            //                                 let pixel_data = bgra_buffer.data;
            //                                 let mut buffer = gst::Buffer::from_slice(pixel_data);
            //                                 buffer
            //                                     .get_mut()
            //                                     .unwrap()
            //                                     .set_pts(Some(gst::ClockTime::from_seconds_f64(pts.as_secs_f64())));

            //                                 let _ = source.push_buffer(buffer);
            //                                 // println!("Write pixel to input")
            //                             }
            //                         })
            //                         .build();

            //                     input_callbacks.push(("input".to_string(), video_input_callback))
            //                 }

            //                 println!(
            //                     "Starting pipeline with description: {}",
            //                     &pipeline_description.join("\n")
            //                 );

            //                 let pipeline = gst::parse::launch(&pipeline_description.join("\n"))
            //                     .unwrap()
            //                     .dynamic_cast::<gst::Pipeline>()
            //                     .unwrap();

            //                 for (name, callback) in input_callbacks {
            //                     let source = pipeline
            //                         .by_name(&name)
            //                         .unwrap()
            //                         .dynamic_cast::<gst_app::AppSrc>()
            //                         .unwrap();
            //                     source.set_callbacks(callback);
            //                 }

            //                 let mut output_file =
            //                     std::fs::File::create(output_path.join("screen_recording.mp4")).unwrap();

            //                 pipeline
            //                     .by_name("output")
            //                     .unwrap()
            //                     .dynamic_cast::<gst_app::AppSink>()
            //                     .unwrap()
            //                     .set_callbacks(
            //                         gst_app::AppSinkCallbacks::builder()
            //                             .new_sample({
            //                                 move |sink| {
            //                                     println!("appsink=output callback!");
            //                                     let sample = sink.pull_sample().unwrap();
            //                                     let buffer = sample.buffer().unwrap();
            //                                     let mapped_buffer = buffer.map_readable().unwrap();
            //                                     let buffer = mapped_buffer.as_slice();
            //                                     println!("Buffer size: {}", buffer.len());

            //                                     output_file.write_all(buffer).unwrap();
            //                                     media_data_clone.lock().unwrap().write_all(buffer).unwrap();

            //                                     Ok(gst::FlowSuccess::Ok)
            //                                 }
            //                             })
            //                             .build(),
            //                     );

            //                 // set pipeline state to playing
            //                 pipeline.set_state(gst::State::Playing).unwrap();

            //                 let pipeline_clone = pipeline.clone();
            //                 std::thread::spawn({
            //                     move || {
            //                         gstreamer_loop(pipeline_clone, |_| false).unwrap();
            //                         println!("Closing pipeline");
            //                     }
            //                 });

            //                 running_pipeline = Some(pipeline);

            //             }
            //             recorder::RecordCommand::Pause => {
            //                 if let Some(pipeline) = &running_pipeline {
            //                     pipeline.set_state(gst::State::Paused).unwrap();
            //                 }
            //             }
            //             recorder::RecordCommand::Resume => {
            //                 if let Some(pipeline) = &running_pipeline {
            //                     pipeline.set_state(gst::State::Playing).unwrap();
            //                 }
            //             }
            //             recorder::RecordCommand::Stop => {
            //                 println!("Before Stop capturing: {}", unsafe{*can_record.load(atomic::Ordering::Relaxed)});
            //                     // can_record = Some(Arc::new(AtomicBool::new(false)));
            //                     can_record.store(Box::into_raw(Box::new(false)), atomic::Ordering::Relaxed);

            //                     println!("After Stop capturing: {}", unsafe{*can_record.load(atomic::Ordering::Relaxed)});

            //                 if let Some(pipeline) = &running_pipeline {
            //                     println!(
            //                         "Before Pipeline yield: {:?}, state: {:?}",
            //                         &pipeline,
            //                         &pipeline.current_state()
            //                     );
            //                     while pipeline.current_state() != gst::State::Null {
            //                         std::thread::yield_now();
            //                     }
            //                     println!(
            //                         "After Pipeline Yield: {:?}, state: {:?}",
            //                         &pipeline,
            //                         &pipeline.current_state()
            //                     );
            //                 }

            //                 {
            //                     unsafe { &mut *recorder.load(atomic::Ordering::Acquire) }.stop_capture();
            //                 }

            //                 println!("Screen recording done");

            //                 let mut media_data = media_data.lock().unwrap();
            //                 media_data.clear();

            //                 running_pipeline = None;
            //             }
            //         }
            //     }
            // });

            Ok(())
        });

    // always listen for mouse and keyboard events when window is unfocused
    let app = app.device_event_filter(tauri::DeviceEventFilter::Always);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
