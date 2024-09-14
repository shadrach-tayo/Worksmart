// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// #[cfg(feature = "unstable_grab")]
use rdev::{listen, Event, EventType};
use serde::{Deserialize, Serialize};
// use std::sync::{atomic::AtomicUsize, Arc};
use tauri::{Manager, WindowEvent};

use worksmart::AppState;

#[derive(Default, Debug, Serialize, Deserialize)]
struct Session {
    // start_time: String,
    // end_time: String,
    id: u128,
    mouse_clicks: usize,
    keystrokes: usize,
    media: Vec<String>,
}

pub fn create_device_query_listener(handle: tauri::AppHandle) {
    std::thread::spawn(|| {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::ButtonPress(rdev::Button::Left) => {
                    if let Err(err) = handle
                        .state::<AppState>()
                        .mouseclick_notifier
                        .as_ref()
                        .unwrap()
                        .send(())
                    {
                        // print error log or send stat to server
                        println!("Error broadcasting keystroke event: {:?}", err);
                    }
                    // todo: broadcase mouse click to any active subscriber
                }
                EventType::KeyPress(_) => {
                    if let Err(err) = handle
                        .state::<AppState>()
                        .key_notifier
                        .as_ref()
                        .unwrap()
                        .send(())
                    {
                        // print error log or send stat to server
                        println!("Error broadcasting keystroke event: {:?}", err);
                    }
                }
                _ => (),
            };

            // Some(event)
        };

        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });
}

#[tokio::main]
async fn main() {
    let (mouseclicks_tx, _) = tokio::sync::broadcast::channel::<()>(1);
    let (keystrokes_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    let app = tauri::Builder::default()
        .manage(AppState {
            mouseclick_notifier: Some(mouseclicks_tx),
            key_notifier: Some(keystrokes_tx),
        })
        .invoke_handler(tauri::generate_handler![
            worksmart::commands::capture_screen
        ])
        .on_window_event(|event| match event.event() {
            WindowEvent::Focused(focused) => {
                if !focused {
                    // event.window().hide().unwrap();
                    println!("Focused: {focused}!");
                }
            }
            WindowEvent::CloseRequested { api, .. } => {
                println!("close request!");
                // if user is in session prevent close or end session first
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .setup(|app| {
            // attach mouse and click broadcaster/subscriber to app state
            // only call when work is in session and close when session has ended
            create_device_query_listener(app.handle());

            Ok(())
        });

    // always listen for mouse and keyboard events when window is unfocused
    let app = app.device_event_filter(tauri::DeviceEventFilter::Always);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
