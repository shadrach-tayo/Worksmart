use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine;
// use dcv_color_primitives::convert_image;
// use gst::prelude::*;
use image::{ImageBuffer, Rgb};
// use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraInfo, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use nokhwa::{native_api_backend, pixel_format, NokhwaError};

use serde::{Deserialize, Serialize};
use tauri::utils::platform;
use tokio::fs;

use crate::compressor;

pub fn get_default_camera() -> crate::Result<CameraInfo> {
    let backend = native_api_backend().unwrap();

    let devices =
        nokhwa::query(backend).map_err(|err| format!("nokhwa::query(backend) error: {:?}", err))?;
    // devices.retain(|camera| camera.human_name() == name.as_str());
    #[allow(clippy::get_first)]
    Ok(devices.get(0).unwrap().to_owned())
}

pub fn create_camera(info: &CameraInfo) -> Result<Camera, NokhwaError> {
    let frame_rate = 30;
    let resolution = Resolution::new(1920, 1080);

    let requested = RequestedFormat::new::<pixel_format::RgbFormat>(
        RequestedFormatType::ClosestIgnoringFormat {
            resolution,
            frame_rate,
        },
    );
    dbg!(&requested);

    Camera::new(info.index().to_owned(), requested)
}

pub fn find_camera(selected_device: &String) -> Result<CameraInfo, String> {
    let backend = native_api_backend().unwrap();
    let devices = nokhwa::query(backend).map_err(|err| err.to_string())?;
    println!("There are {} available cameras.", devices.len());
    dbg!(&devices);

    devices
        .into_iter()
        .find(|device| &device.human_name() == selected_device)
        .ok_or(format!("Cannot find selected device: {}", selected_device))
}

pub fn find_and_create_camera(selected_device: &String) -> Result<(CameraInfo, Camera), String> {
    let info = find_camera(selected_device)?;
    let camera = create_camera(&info).map_err(|err| err.to_string())?;
    dbg!(camera.camera_format());
    dbg!(camera.frame_format());

    Ok((info, camera))
}

/// implement camera controller that handles webcamp capture
/// on init it takes in camera_tx
///

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CameraSnapshotOptions {
    pub save_path: PathBuf,
    // pub id: String,
    pub selected_device: String,
    pub compress: bool,
    pub delay: u64,
}

#[derive(Debug, Clone)]
pub struct CameraController {}

impl CameraController {
    pub async fn take_snapshot(options: CameraSnapshotOptions) -> Result<String, String> {
        let is_granted = nokhwa::nokhwa_check();
        if !is_granted {
            println!("Permission not granted: {is_granted}");
            return Err("Permission required!".into());
        }

        let save_path = options.save_path.clone();

        let _ = tokio::spawn(tokio::time::sleep(Duration::from_secs(5))).await;

        #[cfg(target_os = "macos")]
        {
            let mut cmd = std::process::Command::new(relative_command_path("ffmpeg").unwrap());

            cmd.args(vec!["-ss", "0.5"])
                .args(vec!["-t", "2"])
                .args(vec!["-f", "avfoundation"])
                .args(vec!["-framerate", "30"])
                .args(vec!["-i", &options.selected_device])
                .args(vec!["-vf", "scale=720:-1,setdar=16/9"])
                .args(vec!["-vframes", "1", save_path.to_str().unwrap()]);

            let mut child = cmd
                .spawn()
                .unwrap_or_else(|err| panic!("Ffmpeg command not found {:?}", err));

            match child.wait() {
                Ok(status) if status.success() => {
                    println!("exited with: {status}");
                    if options.compress {
                        compressor::compress_image(
                            save_path.clone(),
                            options.save_path.clone().to_path_buf(),
                        );
                    }
                }
                Ok(status) => eprintln!("exited with: {status}"),
                Err(e) => println!("error attempting to wait: {e}"),
            };

        }

        #[cfg(not(target_os = "macos"))]
        {
            // make the camera
            let (info, mut camera) = find_and_create_camera(&options.selected_device)?;
            dbg!(info);

            camera.open_stream().unwrap();

            // get a frame
            println!(
                "Frame format: {:?}, camera_format: {:?}",
                camera.frame_format(),
                camera.camera_format()
            );
            let frame = camera.frame().unwrap();
            camera.stop_stream().unwrap();
            println!("Captured Single Frame of {}", frame.buffer().len());

            let path = save_path.clone();

            match convert_buffer_to_image(frame.clone()) {
                Ok(image) => {
                    if let Err(err) = image.save(path) {
                        println!("Error saving webcam image {:?}", err);
                    } else {
                        compressor::compress_image(
                            save_path.clone(),
                            options.save_path.clone().to_path_buf(),
                        );
                    }
                }
                Err(err) => {
                    eprintln!("Error saving webcam image {:?}", err);
                }
            };

            // decode into an ImageBuffer
            let decoded = frame.decode_image::<pixel_format::RgbFormat>().unwrap();
            println!("Decoded Frame of {}", decoded.len());

            let path = save_path.clone();

            if let Err(err) = decoded.save(path) {
                eprintln!("Error saving webcam image {:?}", err);
            } else {
                compressor::compress_image(
                    save_path.clone(),
                    options.save_path.clone().to_path_buf(),
                );
            }
        }

        let data = fs::read(options.save_path).await.map_err(|err| err.to_string())?;
        let result = base64::engine::general_purpose::STANDARD.encode(&data);
        Ok(result)
    }
}

fn relative_command_path(command: impl AsRef<Path>) -> crate::Result<PathBuf> {
    match platform::current_exe()?.parent() {
        #[cfg(windows)]
        Some(exe_dir) => Ok(exe_dir.join(command.as_ref()).with_extension("exe")),
        #[cfg(not(windows))]
        Some(exe_dir) => Ok(exe_dir.join(command.as_ref())),
        None => Err("Error::CurrentExeHasNoParent".into()),
    }
}

#[allow(dead_code)]
fn convert_buffer_to_image(
    buffer: nokhwa::Buffer,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
    let Resolution {
        width_x: width,
        height_y: height,
    } = buffer.resolution();
    let mut image_buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);
    let data = buffer.buffer();

    for (y, chunk) in data
        .chunks_exact((width * 2) as usize)
        .enumerate()
        .take(height as usize)
    {
        for (x, pixel) in chunk.chunks_exact(4).enumerate() {
            let [u, y1, v, y2] = [
                pixel[0] as f32,
                pixel[1] as f32,
                pixel[2] as f32,
                pixel[3] as f32,
            ];
            let x = (x * 2) as u32;
            image_buffer.put_pixel(x, y as u32, yuv_to_rgb(y1, u, v));
            image_buffer.put_pixel(x + 1, y as u32, yuv_to_rgb(y2, u, v));
        }
    }

    Ok(image_buffer)
}

//YUV to RGB conversion BT.709
fn yuv_to_rgb(y: f32, u: f32, v: f32) -> Rgb<u8> {
    let r = y + 1.5748 * (v - 128.0);
    let g = y - 0.1873 * (u - 128.0) - 0.4681 * (v - 128.0);
    let b = y + 1.8556 * (u - 128.0);

    Rgb([r as u8, g as u8, b as u8])
}

// fn yuv_to_rgb_bt709(y: f32, u: f32, v: f32) -> Rgb<u8> {
//     let y = y as f32;
//     let u = (u as f32) - 128.0;
//     let v = (v as f32) - 128.0;

//     let r = (y + 1.5748 * v).round().clamp(0.0, 255.0) as u8;
//     let g = (y - 0.187324 * u - 0.468124 * v).round().clamp(0.0, 255.0) as u8;
//     let b = (y + 1.8556 * u).round().clamp(0.0, 255.0) as u8;

//     // (r, g, b)
//     Rgb([r, g, b])
// }
