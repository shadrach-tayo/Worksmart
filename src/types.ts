export interface User {
  name: "Tay";
  token: "jwt-token";
}

export interface Session {
  id: string;
  started_at: string;
  ended_at: string;
}

export interface Configuration {
  capsule_storage_dir: string;
  media_storage_dir: string;
  launch_on_startup: boolean;
  signin_on_launch: boolean;
  track_on_signin: boolean;
  enable_camera: boolean;
  preferences: Preferences;
}

export interface Preferences {
  time_gap_duration_in_seconds: number;
  webcam_delay: number;
}

export interface PermisssionsStatus {
  camera: boolean;
  accessibility: boolean;
  screen_capture: boolean;
}
