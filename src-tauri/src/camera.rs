use nokhwa::{native_api_backend, utils::CameraInfo};

pub fn get_default_camera() -> crate::Result<CameraInfo> {
    let backend = native_api_backend().unwrap();

    let devices =
        nokhwa::query(backend).map_err(|err| format!("nokhwa::query(backend) error: {:?}", err))?;
    // devices.retain(|camera| camera.human_name() == name.as_str());
    #[allow(clippy::get_first)]
    Ok(devices.get(0).unwrap().to_owned())
}
