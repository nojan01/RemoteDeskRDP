import { onCleanup, onMount } from "solid-js";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { listen } from "@tauri-apps/api/event";
import { closeTerminalSession, minimizeTerminalWindow, resizeSshSession, startSshSession, stopSshSession, writeSshSession } from "./api";
import { errMsg, t } from "./i18n";
import type { RemoteProfile } from "./types";
import "@xterm/xterm/css/xterm.css";

export function SshTerminal(props: { profile: RemoteProfile }) {
  let host!: HTMLDivElement;
  let alive = true;
  let terminal: Terminal | undefined;
  let unlistenData: (() => void) | undefined;
  let unlistenEnded: (() => void) | undefined;
  let input: { dispose: () => void } | undefined;
  let resize: ResizeObserver | undefined;
  let autoMinimizeTimer: number | undefined;

  onCleanup(() => {
    alive = false;
    input?.dispose();
    unlistenData?.();
    unlistenEnded?.();
    resize?.disconnect();
    if (autoMinimizeTimer !== undefined) window.clearTimeout(autoMinimizeTimer);
    terminal?.dispose();
    void stopSshSession(props.profile.id).catch(() => undefined);
  });

  onMount(async () => {
    const instance = new Terminal({ cursorBlink: true, fontFamily: "Menlo, monospace", fontSize: 14, theme: { background: "#1b1b1b" } });
    terminal = instance;
    const fit = new FitAddon(); instance.loadAddon(fit); instance.open(host); fit.fit();
    unlistenData = await listen<{ profileId: string; data: number[] }>("ssh-data", (event) => {
      if (event.payload.profileId === props.profile.id) instance.write(new Uint8Array(event.payload.data));
    });
    unlistenEnded = await listen<{ profileId: string }>("ssh-ended", (event) => {
      const isFinishedX11App = props.profile.protocol === "ssh"
        && props.profile.x11Forwarding
        && Boolean(props.profile.x11Command.trim());
      if (event.payload.profileId === props.profile.id && isFinishedX11App) {
        // Der SSH-Prozess ist bereits beendet. Wir schließen nun auch dessen
        // minimiertes Terminalfenster, damit die nächste X11-Verbindung eine
        // neue PTY starten kann statt ein altes Dock-Fenster zu reaktivieren.
        void closeTerminalSession(props.profile.id).catch(() => undefined);
      }
    });
    if (!alive) { unlistenData(); unlistenEnded(); instance.dispose(); return; }
    const started = await startSshSession(props.profile, instance.cols, instance.rows)
      .then(() => true)
      .catch((error) => { instance.writeln(`\r\n${t("terminal.error")}: ${errMsg(error)}`); return false; });
    if (!alive) { void stopSshSession(props.profile.id).catch(() => undefined); return; }
    if (!started) return;
    if (props.profile.protocol === "ssh" && props.profile.x11Forwarding && props.profile.x11Command.trim()) {
      // Der Prozess läuft, bevor das entfernte Programm sein X11-Fenster
      // zeichnet. Ein kurzer Aufschub lässt XQuartz dessen Fenster nach vorn
      // bringen; die Konsole bleibt als Dock-Symbol für Meldungen erreichbar.
      autoMinimizeTimer = window.setTimeout(() => {
        if (alive) void minimizeTerminalWindow().catch(() => undefined);
      }, 600);
    }
    input = instance.onData((data) => void writeSshSession(props.profile.id, Array.from(new TextEncoder().encode(data))).catch(() => undefined));
    resize = new ResizeObserver(() => {
      fit.fit();
      void resizeSshSession(props.profile.id, instance.cols, instance.rows).catch(() => undefined);
    });
    resize.observe(host);
  });
  return <div class="ssh-terminal" ref={host} />;
}
