import { invoke } from "@tauri-apps/api/tauri";
import { Configuration, Session, User } from "../types";

export const check_permissions = async (): Promise<boolean> => {
    return await invoke("permissions_granted");
};

export const request_permissions = async (): Promise<boolean> => {
    return await invoke("request_permissions");
};

export async function record_screen() {
    await invoke("record_screen");
}

export async function start_session(): Promise<Session> {
    return await invoke("start_session");
}

export async function get_session(): Promise<Session | null> {
    return await invoke("get_session");
}

export async function stop_session(): Promise<boolean> {
    return await invoke("stop_session");
}

export async function webcam_capture() {
    const result = await invoke("webcam_capture");
    console.log("result: ", result);
}

export async function login() {
    await invoke("login", { payload: { name: "Tay", token: "jwt-token" } });
}

export async function get_user(): Promise<User | null> {
    return await invoke("get_auth");
}

export async function get_preferences(): Promise<any> {
    return await invoke("get_preferences");
}

export async function set_preferences(
    preferences: Configuration,
): Promise<any> {
    return await invoke("set_preferences", { preferences });
}

export async function update_config() {
    await invoke("update_config");
}

export async function show_window(name: string) {
    await invoke("show_window", { name });
}

export async function hide_window(name: string) {
    await invoke("hide_window", { name });
}

export async function minimize_window(name: string) {
    await invoke("minimize_window", { name });
}
