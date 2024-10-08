import { FormEventHandler, useEffect, useState } from "react";
import { get_preferences, hide_window, set_preferences } from "./ipc";
import "./styles/Settings.css"; // Assuming styles are moved to a separate CSS file named Settings.css
import { Configuration } from "./types";

const Settings = () => {
    const [preferences, setPreferences] = useState<Configuration>();

    const getPreferences = async () => {
        setPreferences(await get_preferences());
    };

    useEffect(() => {
        getPreferences();
    }, []);

    console.log("Preferences", preferences);

    const onSubmit: FormEventHandler = async (evt) => {
        evt.preventDefault();
        let form = evt?.target as HTMLFormElement;

        const config = {
            ...preferences,
            launch_on_startup: form["launchOnStartup"].checked, //  === "on" ? true : false
            signin_on_launch: form["signInOnStartup"].checked, //  === "on" ? true : false
            track_on_signin: form["trackOnSignin"].checked, //  === "on" ? true : false
            enable_camera: form["enableCamera"].checked, //  === "on" ? true : false
        } as Configuration;
        console.log("values", config);
        await set_preferences(config);
        setPreferences(config);
    };

    return (
        <div data-tauri-drag-region className="settings-card p-4">
            <h3 className="mb-4">Settings</h3>
            <hr />
            <form onSubmit={onSubmit}>
                <div className="row">
                    <div className="col-12 col-md-6">
                        <div className="mb-3 form-check">
                            <input
                                type="checkbox"
                                className="form-check-input"
                                id="launchOnStartup"
                                name="launchOnStartup"
                                checked={
                                    preferences?.launch_on_startup ?? false
                                }
                                onChange={(evt) =>
                                    setPreferences({
                                        ...(preferences as Configuration),
                                        launch_on_startup: evt.target.checked,
                                    })
                                }
                            />
                            <label
                                className="form-check-label"
                                htmlFor="launchOnStartup"
                            >
                                Launch at system startup
                            </label>
                        </div>

                        <div className="mb-3 form-check">
                            <input
                                type="checkbox"
                                className="form-check-input"
                                id="signInOnStartup"
                                name="signInOnStartup"
                                checked={preferences?.signin_on_launch ?? false}
                                onChange={(evt) =>
                                    setPreferences({
                                        ...(preferences as Configuration),
                                        signin_on_launch: evt.target.checked,
                                    })
                                }
                            />
                            <label
                                className="form-check-label"
                                htmlFor="signInOnStartup"
                            >
                                Sign in on launch
                            </label>
                        </div>

                        <div className="form-check">
                            <input
                                type="checkbox"
                                className="form-check-input"
                                id="trackOnSignin"
                                name="trackOnSignin"
                                checked={preferences?.track_on_signin ?? false}
                                onChange={(evt) =>
                                    setPreferences({
                                        ...(preferences as Configuration),
                                        track_on_signin: evt.target.checked,
                                    })
                                }
                            />
                            <label
                                className="form-check-label"
                                htmlFor="trackOnSignin"
                            >
                                Start tracking on sign in
                            </label>
                        </div>
                    </div>
                </div>
                <hr />
                <div className="row">
                    <div className="col-6 col-md-6">
                        <div className="mb-4">
                            <img
                                src="https://placehold.co/150x175@3x/FFFFFF/png"
                                alt="Camera Preview"
                                className="camera-preview mb-3"
                            />
                        </div>
                    </div>

                    <div className="col-6 col-md-6">
                        <div className="mb-3">
                            <div className="mb-4 form-check">
                                <input
                                    type="checkbox"
                                    className="form-check-input"
                                    id="enableCamera"
                                    name="enableCamera"
                                    checked={
                                        preferences?.enable_camera ?? false
                                    }
                                    onChange={(evt) =>
                                        setPreferences({
                                            ...(preferences as Configuration),
                                            enable_camera: evt.target.checked,
                                        })
                                    }
                                />
                                <label
                                    className="form-check-label"
                                    htmlFor="enableCamera"
                                >
                                    Enable Camera
                                </label>
                            </div>

                            <label
                                htmlFor="cameraDriver"
                                className="form-label"
                            >
                                Camera Driver
                            </label>
                            <select className="form-select" id="cameraDriver">
                                <option selected>FaceTime HD Camera</option>
                                <option>Logitech Webcam</option>
                                <option>External Camera</option>
                            </select>
                        </div>

                        <div className="mb-4 d-flex align-items-center gap-2">
                            <label htmlFor="delaySelect" className="form-label">
                                Delay (seconds)
                            </label>
                            <select className="form-select" id="delaySelect">
                                <option selected>3 secs</option>
                                <option>5 secs</option>
                                <option>10 secs</option>
                            </select>
                        </div>

                        <button
                            type="button"
                            className="btn btn-outline-light mb-4"
                        >
                            Camera Test
                        </button>
                    </div>
                </div>
                <hr />
                <div className="row">
                    <div className="col-12 col-md-12">
                        <div className="d-flex justify-content-between">
                            <button
                                type="button"
                                className="btn btn-outline-light"
                                onClick={() => hide_window("settings")}
                            >
                                Cancel
                            </button>
                            <button type="submit" className="btn btn-custom">
                                Save
                            </button>
                        </div>
                    </div>
                </div>
            </form>
        </div>
    );
};

export default Settings;
