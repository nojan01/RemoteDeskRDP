import { invoke } from "@tauri-apps/api/core";
import type { RemoteProfile } from "./types";

export const listProfiles = () => invoke<RemoteProfile[]>("list_profiles");
export const saveProfile = (profile: RemoteProfile) =>
  invoke<RemoteProfile>("save_profile", { profile });
export const deleteProfile = (id: string) => invoke<void>("delete_profile", { id });
export const connectProfile = (profile: RemoteProfile) =>
  invoke<void>("connect_profile", { profile });
export const startVncSession = (profile: RemoteProfile) =>
  invoke<{ websocketUrl: string }>("start_vnc_session", { profile });
export const startSshSession = (profile: RemoteProfile, columns: number, rows: number) =>
  invoke<void>("start_ssh_session", { profile, columns, rows });
export const writeSshSession = (profileId: string, data: number[]) =>
  invoke<void>("write_ssh_session", { profileId, data });
export const resizeSshSession = (profileId: string, columns: number, rows: number) =>
  invoke<void>("resize_ssh_session", { profileId, columns, rows });
export const stopSshSession = (profileId: string) =>
  invoke<void>("stop_ssh_session", { profileId });
export const closeTerminalSession = (profileId: string) =>
  invoke<void>("close_terminal_session", { profileId });
export const minimizeTerminalWindow = () =>
  invoke<void>("minimize_terminal_window");
export const loadPassword = (profileId: string) =>
  invoke<string | null>("load_password", { profileId });
export const savePassword = (profileId: string, password: string) =>
  invoke<void>("save_password", { profileId, password });
export const forgetPassword = (profileId: string) =>
  invoke<void>("forget_password", { profileId });
export const loadGatewayPassword = (profileId: string) =>
  invoke<string | null>("load_gateway_password", { profileId });
export const saveGatewayPassword = (profileId: string, password: string) =>
  invoke<void>("save_gateway_password", { profileId, password });
export const forgetGatewayPassword = (profileId: string) =>
  invoke<void>("forget_gateway_password", { profileId });
export const takePendingLink = () => invoke<string | null>("take_pending_link");
export const minimizeWindow = () => invoke<void>("minimize_window");
