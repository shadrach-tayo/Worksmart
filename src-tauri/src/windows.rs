use tauri::{AppHandle, Manager};

pub fn show_login(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("login") {
        auth_window.show().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn close_login(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("login") {
        auth_window.close().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn show_tracker(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("track") {
        auth_window.show().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn close_tracker(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("track") {
        auth_window.close().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn show_settings(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("settings") {
        auth_window.show().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn close_settings(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("settings") {
        auth_window.close().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn show_timecard(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("timecard") {
        auth_window.show().unwrap();
        return Ok(());
    }

    Ok(())
}

pub fn close_timecard(app: &AppHandle) -> crate::Result<()> {
    if let Some(auth_window) = app.get_window("timecard") {
        auth_window.close().unwrap();
        return Ok(());
    }

    Ok(())
}
