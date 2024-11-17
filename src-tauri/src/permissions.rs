// use core_graphics::access::ScreenCaptureAccess;
// use nokhwa::{nokhwa_check, nokhwa_initialize};
use serde::{Deserialize, Serialize};

// #[cfg(target_os = "macos")]
// use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};

pub use nokhwa_bindings_macos::{AVAuthorizationStatus, AVMediaType};

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    // fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
}

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
        println!("CheckAvPermission: {}",check_av_permission(AVMediaType::Video));
        PermisssionsStatus {
            camera: check_av_permission(AVMediaType::Video),
            accessibility: check_accessibility_permission(),
            screen_capture: scap::has_permission()
        }
    }

    pub fn request_permission(_type: PermissionType) {
        println!("request permission");
        open_permission_settings(_type.clone());
        dbg!(&_type);
        match _type {
            PermissionType::Accessibility => {},
            PermissionType::Camera => {
                // #[cfg(target_os = "macos")]
                // nokhwa::nokhwa_initialize(|granted| {
                //     println!("Camera permission granted: {granted}");
                // });
                request_av_permission(AVMediaType::Video);
            },
            PermissionType::ScreenCapture => {
                scap::request_permission();
            },
        }
    }
}

pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // For non-macOS platforms, assume permission is granted
        // OSPermissionStatus::NotNeeded
        false
    }
}

#[cfg(target_os = "macos")]
pub fn check_av_permission(media_type: AVMediaType) -> bool {
    use objc::*;

    let cls = objc::class!(AVCaptureDevice);
    let status: AVAuthorizationStatus =
        unsafe { msg_send![cls, authorizationStatusForMediaType:media_type.into_ns_str()] };
    match status {
        AVAuthorizationStatus::NotDetermined => false,
        AVAuthorizationStatus::Authorized => true,
        _ => false,
    }
}


#[cfg(target_os = "macos")]
fn request_av_permission(media_type: AVMediaType) {
    use objc::{runtime::*, *};
    use tauri_nspanel::block::ConcreteBlock;

    let callback = move |_: BOOL| {};
    let cls = class!(AVCaptureDevice);
    let objc_fn_block: ConcreteBlock<(BOOL,), (), _> = ConcreteBlock::new(callback);
    let objc_fn_pass = objc_fn_block.copy();
    unsafe {
        let _: () = msg_send![cls, requestAccessForMediaType:media_type.into_ns_str() completionHandler:objc_fn_pass];
    };
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
