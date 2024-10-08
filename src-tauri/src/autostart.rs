// use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder};

fn handle() -> Result<AutoLaunch, String> {
    let app_path = std::env::current_exe()
        .map_err(|err| err.to_string())?
        .into_os_string()
        .to_str()
        .unwrap()
        .to_owned();

    let autolaunc = AutoLaunchBuilder::new()
        .set_app_name("Notor")
        .set_app_path(&app_path)
        .set_use_launch_agent(true)
        .set_args(&["--hidden"])
        .build()
        .map_err(|err| err.to_string())?;

    Ok(autolaunc)
}

pub fn update(flag: bool) -> anyhow::Result<()> {
    let handle = handle();

    let handle = handle.unwrap();

    if handle.is_enabled()? == flag {
        return Ok(());
    }

    if flag {
        handle.enable()?;
    } else {
        handle.disable()?;
    }

    Ok(())
}
