import "./styles/Settings.css"; // Assuming styles are moved to a separate CSS file named Settings.css

const Settings = () => {
    return (
        <div data-tauri-drag-region className="settings-card p-4">
            <h3 className="mb-4">Settings</h3>
            <hr />
            <form>
                <div className="row">
                    <div className="col-12 col-md-6">
                        <div className="mb-3 form-check">
                            <input
                                type="checkbox"
                                className="form-check-input"
                                id="startupCheck"
                                defaultChecked
                            />
                            <label
                                className="form-check-label"
                                htmlFor="startupCheck"
                            >
                                Launch at system startup
                            </label>
                        </div>

                        <div className="mb-3 form-check">
                            <input
                                type="checkbox"
                                className="form-check-input"
                                id="signinCheck"
                            />
                            <label
                                className="form-check-label"
                                htmlFor="signinCheck"
                            >
                                Sign in on launch
                            </label>
                        </div>

                        <div className="form-check">
                            <input
                                type="checkbox"
                                className="form-check-input"
                                id="trackSigninCheck"
                                defaultChecked
                            />
                            <label
                                className="form-check-label"
                                htmlFor="trackSigninCheck"
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
                                    id="enableCameraCheck"
                                    defaultChecked
                                />
                                <label
                                    className="form-check-label"
                                    htmlFor="enableCameraCheck"
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
