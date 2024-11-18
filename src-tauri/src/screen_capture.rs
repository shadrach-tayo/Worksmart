use std::path::PathBuf;

use active_win_pos_rs::{get_active_window, ActiveWindow};
use xcap::Monitor;

use crate::get_current_datetime;

pub fn get_focused_window() -> Option<ActiveWindow> {
    match get_active_window() {
            Ok(active_window) => {
                // println!("active window: {:#?}", active_window);
                Some(active_window)
            },
            Err(()) => {
                println!("error occurred while getting the active window");
                // Err("Could not retrieve active app window".to_string())
                None
            }
        }
}


#[derive(Debug, Clone)]
pub struct ScreenCapture {}

pub struct ScreenshotOptions {
    pub output: PathBuf,
    // pub window: String,
}

impl ScreenCapture {
    pub async fn take_screenshot(options: ScreenshotOptions) -> crate::Result<()> {
        let monitors = Monitor::all().unwrap();
        let window = match get_focused_window() {
            Some(w) => w.app_name,
            None => "".to_owned()
        };

        for monitor in monitors {
            let image = monitor.capture_image().unwrap();

            let window_name = if window.is_empty() {
                monitor.name()
            } else {
                &window
            };

            let img_path = options.output.clone().join(format!(
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
            };
        }

        Ok(())
    }
}
