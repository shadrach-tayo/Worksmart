use tauri::{AppHandle, Manager, PhysicalPosition, Window, Wry};

pub enum AppWindow {
    Login,
    Track,
    TimeCard,
    Settings,
    Permissions
}

impl AppWindow {
    pub fn label(&self) -> String {
        match self {
            AppWindow::Login => "login".to_string(),
            AppWindow::Track => "track".to_string(),
            AppWindow::TimeCard => "time-card".to_string(),
            AppWindow::Settings => "settings".to_string(),
            AppWindow::Permissions => "permissions".to_string(),
        }
    }

    pub fn title(&self) -> String {
        match self {
            AppWindow::Login => "login".to_string(),
            AppWindow::Track => "track".to_string(),
            AppWindow::TimeCard => "time card".to_string(),
            AppWindow::Settings => "settings".to_string(),
            AppWindow::Permissions => "permissions".to_string(),
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "login" => AppWindow::Login,
            "track" => AppWindow::Track,
            "time-card" => AppWindow::TimeCard,
            "settings" => AppWindow::Settings,
            "permissions" => AppWindow::Permissions,
            _ => unreachable!("unknown window: {label}")
        }
    }

    pub fn get(&self, app: &AppHandle<Wry>) -> Option<Window> {
        let label = self.label();
        app.get_window(&label)
    }

    pub fn show(&self, app: &AppHandle<Wry>) -> tauri::Result<Window<Wry>> {
        let label = self.label();

        if let Some(window) = app.get_window(&label) {
            window.show().ok();
            window.set_focus().ok();
            return Ok(window);
        }

        Ok(match self {
            AppWindow::Login => {
                tauri::WindowBuilder::new(app, label, tauri::WindowUrl::App("/login".into()))
                    .center()
                    .title(self.title())
                    .hidden_title(true)
                    .maximizable(false)
                    .minimizable(false)
                    .maximized(false)
                    .resizable(false)
                    .inner_size(380.0, 355.0)
                    .transparent(true)
                    .decorations(false)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .theme(Some(tauri::Theme::Dark))
                    .build()?
            },
            AppWindow::Track => {
                let window = tauri::WindowBuilder::new(app, label, tauri::WindowUrl::App("/track".into()))
                    .center()
                    .title(self.title())
                    .hidden_title(true)
                    .maximizable(false)
                    .maximized(false)
                    .resizable(false)
                    .inner_size(463.0, 220.0)
                    .transparent(true)
                    .decorations(false)
                    // .always_on_top(true)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .theme(Some(tauri::Theme::Dark))
                    .build()?;

                if let Some(monitor) = window.current_monitor()? {
                    window.set_position(
                        // bottom right position
                        PhysicalPosition {
                                x: (monitor.size().width as f64) - 463.0 * monitor.scale_factor(), // right
                                y: (monitor.size().height as f64) - (220.0 * monitor.scale_factor()) - 50.0 // bottom
                        }
                    )?;
                }

                window
            },
            AppWindow::TimeCard => {
                let window = tauri::WindowBuilder::new(app, label, tauri::WindowUrl::App("/timecard".into()))
                    .center()
                    .title(self.title())
                    .hidden_title(true)
                    .maximizable(false)
                    .minimizable(false)
                    .maximized(false)
                    .resizable(false)
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(true)
                    .inner_size(503.0, 183.0)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .theme(Some(tauri::Theme::Dark))
                    .build()?;

                if let Some(monitor) = window.current_monitor()? {
                    window.set_position(
                        // top right position
                        PhysicalPosition {
                                x: (monitor.size().width as f64) - 503.0 * monitor.scale_factor(), // right
                                y: 100.0 // top
                        }
                    )?;
                }

                window
            },
            AppWindow::Settings => {
                tauri::WindowBuilder::new(app, label, tauri::WindowUrl::App("/settings".into()))
                    .center()
                    .title(self.title())
                    .hidden_title(true)
                    .maximizable(false)
                    .minimizable(false)
                    .maximized(false)
                    .resizable(false)
                    .inner_size(460.0, 620.0)
                    .transparent(true)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .theme(Some(tauri::Theme::Dark))
                    .build()?
            },
            AppWindow::Permissions => {
                tauri::WindowBuilder::new(app, label, tauri::WindowUrl::App("/permissions".into()))
                    .center()
                    .title(self.title())
                    .hidden_title(true)
                    .maximizable(false)
                    .minimizable(false)
                    .maximized(false)
                    .resizable(false)
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(true)
                    .inner_size(400.0, 340.0)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .theme(Some(tauri::Theme::Dark))
                    .build()?
            },
        })
    }

    pub fn close(&self, app: &AppHandle<Wry>) {
        if let Some(window) = app.get_window(&self.label()) {
            tauri::async_runtime::spawn(async move {
                window.close().ok();
            });
        }
    }
}

pub fn show_login(app: &AppHandle) {
    (AppWindow::Login).show(app).ok();
}

pub fn close_login(app: &AppHandle) {
    AppWindow::Login.close(app)
}

pub fn show_tracker(app: &AppHandle) {
    (AppWindow::Track).show(app).ok();
}

pub fn close_tracker(app: &AppHandle) {
    (AppWindow::Track).close(app);
}

pub fn show_settings(app: &AppHandle) {
    (AppWindow::Settings).show(app).ok();
}

pub fn close_settings(app: &AppHandle) {
    (AppWindow::Settings).close(app);
}

pub fn show_timecard(app: &AppHandle) {
    (AppWindow::TimeCard).show(app).ok();
}

pub fn close_timecard(app: &AppHandle) {
    (AppWindow::TimeCard).close(app);
}

pub fn show_permission(app: &AppHandle) {
    (AppWindow::Permissions).show(app).ok();
}

pub fn close_permission(app: &AppHandle) {
    (AppWindow::Permissions).close(app);
}

// pub fn show_window(app: &AppHandle, name: String) -> crate::Result<()> {
//     if let Some(window) = app.get_window(&name) {
//         window.show().unwrap();
//         return Ok(());
//     }

//     Ok(())
// }

// pub fn hide_window(app: &AppHandle, name: String) -> crate::Result<()> {
//     if let Some(window) = app.get_window(&name) {
//         window.close().unwrap();
//         return Ok(());
//     }

//     Ok(())
// }


// {
//   "title": "main",
//   "width": 600,
//   "height": 300,
//   "resizable": false,
//   "visible": false
// },
// {
//   "label": "login",
//   "fullscreen": false,
//   "resizable": false,
//   "maximizable": false,
//   "title": "Login",
//   "width": 420,
//   "height": 420,
//   "decorations": false,
//   "alwaysOnTop": false,
//   "contentProtected": true,
//   "visible": false,
//   "url": "/login"
// },
// {
//   "label": "settings",
//   "fullscreen": false,
//   "resizable": false,
//   "maximizable": false,
//   "title": "Settings",
//   "decorations": false,
//   "alwaysOnTop": false,
//   "contentProtected": true,
//   "visible": false,
//   "url": "/settings",
//   "width": 460,
//   "height": 620
// },
// {
//   "label": "timecard",
//   "fullscreen": false,
//   "resizable": false,
//   "maximizable": false,
//   "title": "Timecard",
//   "decorations": false,
//   "alwaysOnTop": false,
//   "contentProtected": true,
//   "visible": false,
//   "url": "/timecard",
//   "width": 503,
//   "height": 233
// },
// {
//   "label": "track",
//   "fullscreen": false,
//   "resizable": false,
//   "maximizable": false,
//   "title": "Track",
//   "decorations": false,
//   "alwaysOnTop": false,
//   "contentProtected": true,
//   "visible": false,
//   "url": "/track",
//   "width": 463,
//   "height": 220
// }
//
