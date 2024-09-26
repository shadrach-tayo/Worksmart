import { invoke } from "@tauri-apps/api/tauri";

export const check_permissions = async (): Promise<boolean> => {
  return await invoke("permissions_granted");
};

export const request_permissions = async (): Promise<boolean> => {
  return await invoke("request_permissions");
};

export async function record_screen() {
  await invoke("record_screen");
}

export async function start_session(): Promise<boolean> {
  return await invoke("start_session");
}

export async function stop_session(): Promise<boolean> {
  return await invoke("stop_session");
}

export async function update_config() {
  await invoke("update_config");
}

export async function webcam_capture() {
  const result = await invoke("webcam_capture");
  console.log("result: ", result);
}
