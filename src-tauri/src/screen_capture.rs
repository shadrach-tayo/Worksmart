use active_win_pos_rs::{get_active_window, ActiveWindow};

pub fn get_focused_window() -> Option<ActiveWindow> {
    match get_active_window() {
            Ok(active_window) => {
                println!("active window: {:#?}", active_window);
                Some(active_window)
            },
            Err(()) => {
                println!("error occurred while getting the active window");
                // Err("Could not retrieve active app window".to_string())
                None
            }
        }
}
