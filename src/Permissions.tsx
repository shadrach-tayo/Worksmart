import { useEffect, useState } from "react";
import "./styles/Permissions.css";
import { PermisssionsStatus } from "./types";
import {
  get_permission_status,
  on_permissions_granted,
  request_accessibility_permissions,
  request_camera_permissions,
  request_screen_capture_permissions,
} from "./ipc";

export default function PermissionRequest() {
  const [status, setStatus] = useState<PermisssionsStatus>(() => ({
    accessibility: false,
    camera: false,
    screen_capture: false,
  }));

  const getStatus = async () => {
    const status = await get_permission_status();
    setStatus(status);
  };

  useEffect(() => {
    getStatus();
  }, []);

  return (
    <div
      style={{
        backgroundColor: "#1a1a1a",
        color: "#fff",
        fontFamily: "Arial, sans-serif",
      }}
      className="permission"
    >
      <div className="permission-card" data-tauri-drag-region>
        <div className="permission-icon">
          <i className="bi bi-display"></i>
        </div>
        <div className="permission-header">Permission Request</div>
        <p className="permission-subtext font-semibold">
          Each permission you grant allows us to enhance your workflow and
          support collaboration
        </p>
        <form>
          <div className="pl-0 flex items-center justify-start gap-2">
            <label className="ml-0" htmlFor="cameraUse">
              Allow camera use for photos
            </label>
            <div className="form-switch">
              <input
                className="form-check-input ml-0"
                type="checkbox"
                id="cameraUse"
                checked={status.camera}
                disabled={status.camera}
                onChange={(_) => {
                  request_camera_permissions();
                  getStatus();
                }}
              />
            </div>
          </div>
          <div className="pl-0 flex items-center justify-start gap-2">
            <label className="ml-0" htmlFor="cameraUse">
              keyboard and mouse accessibility
            </label>
            <div className="form-switch">
              <input
                className="form-check-input ml-0"
                type="checkbox"
                id="cameraUse"
                checked={status.accessibility}
                disabled={status.accessibility}
                onChange={(_) => {
                  request_accessibility_permissions();
                  getStatus();
                }}
              />
            </div>
          </div>
          <div className="flex items-center justify-start gap-2">
            <label className="form-check-label" htmlFor="microphoneUse">
              For Screen capture and recording
            </label>
            <div className="form-switch">
              <input
                className="form-check-input"
                type="checkbox"
                id="microphoneUse"
                checked={status.screen_capture}
                disabled={status.screen_capture}
                onChange={(_) => {
                  request_screen_capture_permissions();
                  getStatus();
                }}
              />
            </div>
          </div>
          <button
            type="button"
            className="btn btn-close-custom"
            onClick={on_permissions_granted}
            disabled={
              !(status.accessibility && status.camera && status.screen_capture)
            }
          >
            Continue
          </button>
        </form>
      </div>
    </div>
  );
}
