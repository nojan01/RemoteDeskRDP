/**
 * Software-Aktualisierung über den Tauri-Updater (wie in DualBeam).
 *
 * Beim Start wird still geprüft und nur bei einer neuen Version nachgefragt.
 * Über das Menü angestoßen meldet sich die Prüfung auch, wenn alles aktuell
 * ist, und zeigt Fehler an. Download und Austausch laufen im Rust-Backend.
 */
import { check } from "@tauri-apps/plugin-updater";
import { activeSessionCount, appVersion, restartApplication } from "./api";
import { confirmDialog, infoDialog } from "./dialogs";
import { t } from "./i18n";

/** Fortschrittsmeldung als Schlüssel samt Parametern für die Zustandszeile. */
export type UpdateReporter = (key: string | null, params?: Record<string, string | number>) => void;

/** Verhindert zwei gleichzeitige Prüfungen, etwa Start und Menübefehl. */
let running = false;

export async function checkForUpdates(interactive = false, report: UpdateReporter = () => {}): Promise<void> {
  if (running) return;
  running = true;
  try {
    if (interactive) report("update.checking");
    const update = await check();
    if (!update) {
      report(null);
      if (interactive) await infoDialog(t("update.upToDate", { version: await appVersion() }));
      return;
    }
    report(null);
    const question = `${t("update.available", { version: update.version, current: update.currentVersion })}\n\n${t("update.question")}`;
    if (!(await confirmDialog(question))) return;

    // SSH- und VNC-Sitzungen laufen im Prozess, RDP-Sitzungen werden beim
    // Neustart beendet – das darf nicht unangekündigt passieren.
    const open = await activeSessionCount().catch(() => 0);
    if (open > 0 && !(await confirmDialog(t("update.sessionsOpen", { count: open })))) return;

    let total = 0;
    let loaded = 0;
    report("update.preparing");
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength ?? 0;
          break;
        case "Progress":
          loaded += event.data.chunkLength;
          if (total > 0) report("update.downloading", { percent: Math.min(100, Math.round((loaded / total) * 100)) });
          break;
        case "Finished":
          report("update.installing");
          break;
      }
    });
    report("update.done", { version: update.version });
    await infoDialog(t("update.done", { version: update.version }));
    await restartApplication();
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err);
    console.error("Update-Prüfung fehlgeschlagen:", detail);
    report(interactive ? "err.update.failed" : null, { error: detail });
    if (interactive) await infoDialog(t("err.update.failed", { error: detail }), "error");
  } finally {
    running = false;
  }
}
