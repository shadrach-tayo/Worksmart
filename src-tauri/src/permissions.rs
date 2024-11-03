use core_graphics::access::ScreenCaptureAccess;
use nokhwa::{nokhwa_check, nokhwa_initialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum PermissionType {
    Camera,
    Accessibility,
    ScreenCapture
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermisssionsStatus {
    pub camera: bool,
    pub accessibility: bool,
    pub screen_capture: bool,
}

impl PermisssionsStatus {
    pub fn required_granted(&self) -> bool {
        self.camera && self.accessibility && self.screen_capture
    }

    pub fn get_status() -> Self {
        PermisssionsStatus {
            camera: nokhwa_check(),
            accessibility: scap::has_permission(),
            screen_capture:  ScreenCaptureAccess::default().preflight(),
        }
    }

    pub fn request_permission(_type: PermissionType) {
        println!("request permission");
        open_permission_settings(_type.clone());
        dbg!(&_type);
        match _type {
            PermissionType::Accessibility => {scap::request_permission();},
            PermissionType::Camera => {
                nokhwa_initialize(|granted| {
                    println!("Camera permission granted: {granted}");
                });
            },
            PermissionType::ScreenCapture => {ScreenCaptureAccess::default().request();},
        }
    }
}


// #[tauri::command(async)]
pub fn open_permission_settings(permission: PermissionType) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        match permission {
            PermissionType::ScreenCapture => {
                Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                    .spawn()
                    .expect("Failed to open Screen Recording settings");
            }
            PermissionType::Camera => {
                Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Camera")
                    .spawn()
                    .expect("Failed to open Camera settings");
            }
            PermissionType::Accessibility => {
                Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                    .spawn()
                    .expect("Failed to open Accessibility settings");
            }
        }
    }

    // todo: run bash commands to open settings on windows
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        match permission {
            PermissionType::ScreenCapture => {
                Command::new("open")
                    .arg("start ms-settings:privacy")
                    .spawn()
                    .expect("Failed to open Screen Recording settings");
            }
            PermissionType::Camera => {
                Command::new("open")
                    .arg("start ms-settings:privacy")
                    .spawn()
                    .expect("Failed to open Camera settings");
            }
            PermissionType::Accessibility => {
                Command::new("open")
                    .arg("start ms-settings:easeofaccess")
                    .spawn()
                    .expect("Failed to open Accessibility settings");
            }
        }
    }

    // // todo: run bash commands to open settings on linux
    // #[cfg(target_os = "linux")]
    // {
    //     use std::process::Command;

    //     match permission {
    //         PermissionType::ScreenCapture => {
    //             Command::new("open")
    //                 .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
    //                 .spawn()
    //                 .expect("Failed to open Screen Recording settings");
    //         }
    //         PermissionType::Camera => {
    //             Command::new("open")
    //                 .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Camera")
    //                 .spawn()
    //                 .expect("Failed to open Camera settings");
    //         }
    //         PermissionType::Accessibility => {
    //             Command::new("open")
    //                 .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
    //                 .spawn()
    //                 .expect("Failed to open Accessibility settings");
    //         }
    //     }
    // }
}
