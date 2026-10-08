/**
 * Native Rückfragen und Meldungen.
 *
 * WKWebView zeigt unter Tauri weder `alert()` noch `confirm()` an; `confirm()`
 * liefert dort stets `false`. Deshalb laufen alle Dialoge über das Dialog-Plugin.
 */
import { ask, message } from "@tauri-apps/plugin-dialog";

const TITLE = "RemoteDeskRDP";

export async function confirmDialog(text: string): Promise<boolean> {
  try {
    return await ask(text, { title: TITLE, kind: "warning" });
  } catch (error) {
    console.error("Rückfrage fehlgeschlagen:", error);
    return false;
  }
}

export async function infoDialog(text: string, kind: "info" | "error" = "info"): Promise<void> {
  try {
    await message(text, { title: TITLE, kind });
  } catch (error) {
    console.error("Meldung fehlgeschlagen:", error);
  }
}
