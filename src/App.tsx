import { For, Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import RFB from "@novnc/novnc";
import { closeTerminalSession, connectProfile, deleteProfile, forgetGatewayPassword, forgetPassword, listProfiles, loadGatewayPassword, loadPassword, minimizeWindow, saveGatewayPassword, savePassword, saveProfile, startVncSession, takePendingLink } from "./api";
import { listen } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import { emptyProfile, resolutionPresets, type ColorDepth, type DisplayMode, type Protocol, type RemoteProfile, type ResizeBehavior, type SharedFolder } from "./types";
import { cycleLangMode, errMsg, getLangMode, langIcon, t } from "./i18n";
import { cycleThemeMode, getThemeMode, themeIcon } from "./theme";
import { SshTerminal } from "./SshTerminal";

type Field = keyof RemoteProfile;

/** Die Zustandszeile hält einen Schlüssel statt fertigem Text. Nur so wechselt
 *  auch eine stehende Meldung mit, wenn der Nutzer die Sprache umschaltet.
 *  `error` bleibt roh und wird erst beim Zeichnen übersetzt. */
type Status = { key: string; name?: string; error?: unknown };

/** Reihenfolge der Hilfeabschnitte; die Texte stehen in den Wörterbüchern. */
const helpSections = [
  "targets", "ports", "vnc", "ssh", "sftp", "mosh", "gateway", "udp", "reconnect", "display", "files", "folders",
  "limits", "printer", "smartcard", "video", "security", "appearance", "dualbeam",
] as const;

/** Reihenfolge der Lizenzabschnitte; die Texte stehen in den Wörterbüchern. */
const licenseSections = ["mit", "modules", "novnc", "source", "freerdp"] as const;

function readPort(value: string, fallback: number) {
  const number = Number(value);
  return Number.isInteger(number) && number > 0 && number < 65536 ? number : fallback;
}

/** FreeRDP lehnt ungerade Kantenlängen ab, deshalb wird hier gerundet. */
function readPixels(value: string, fallback: number) {
  const number = Math.round(Number(value) / 2) * 2;
  return Number.isFinite(number) && number >= 640 && number <= 7680 ? number : fallback;
}

export function App() {
  // Ein VNC-Fenster lädt dieselbe Oberfläche mit einer Profil-ID in der URL.
  // Es zeigt ausschließlich die Sitzung; das Hauptfenster bleibt Profilzentrale.
  const vncProfileId = new URLSearchParams(window.location.search).get("vnc");
  const terminalProfileId = new URLSearchParams(window.location.search).get("terminal") ?? new URLSearchParams(window.location.search).get("ssh");
  const isVncWindow = vncProfileId !== null;
  const isTerminalWindow = terminalProfileId !== null;
  const [profiles, setProfiles] = createSignal<RemoteProfile[]>([]);
  const [current, setCurrent] = createSignal<RemoteProfile>(emptyProfile());
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [status, setStatus] = createSignal<Status>({ key: "state.ready" });
  const [helpOpen, setHelpOpen] = createSignal(false);
  const [licenseOpen, setLicenseOpen] = createSignal(false);
  const [dirty, setDirty] = createSignal(false);
  const [password, setPassword] = createSignal("");
  const [revealPassword, setRevealPassword] = createSignal(false);
  const [gatewayPassword, setGatewayPassword] = createSignal("");
  const [revealGatewayPassword, setRevealGatewayPassword] = createSignal(false);
  const [vncSession, setVncSession] = createSignal<{ profile: RemoteProfile; websocketUrl: string } | null>(null);
  const [vncCredentialsNeeded, setVncCredentialsNeeded] = createSignal(false);
  const [vncPassword, setVncPassword] = createSignal("");
  const [vncSavePassword, setVncSavePassword] = createSignal(false);
  let vncDisplay: HTMLDivElement | undefined;
  let vncRfb: any;
  // Ein später Keychain-Lesevorgang darf nie die Daten eines inzwischen
  // ausgewählten Profils überschreiben.
  let secretLoadGeneration = 0;

  /** Setzt die Meldung erst beim Zeichnen zusammen – dadurch folgt sie der Sprache. */
  const statusText = () => {
    const s = status();
    return t(s.key, {
      name: s.name ?? "",
      error: s.error === undefined ? "" : errMsg(s.error),
    });
  };

  const loadSecret = async (profileId: string) => {
    const generation = ++secretLoadGeneration;
    const [sessionPassword, gatewaySecret] = await Promise.all([
      loadPassword(profileId).catch(() => null),
      loadGatewayPassword(profileId).catch(() => null),
    ]);
    if (generation !== secretLoadGeneration || selectedId() !== profileId) return;
    setPassword(sessionPassword ?? "");
    setGatewayPassword(gatewaySecret ?? "");
  };

  const startVnc = async (profile: RemoteProfile) => {
    setVncCredentialsNeeded(false); setVncPassword(""); setVncSavePassword(Boolean(password()));
    const endpoint = await startVncSession(profile);
    setVncSession({ profile, websocketUrl: endpoint.websocketUrl });
  };
  const closeVnc = () => {
    setVncCredentialsNeeded(false); setVncPassword(""); vncRfb?.disconnect(); vncRfb = undefined;
    setVncSession(null); setStatus({ key: "state.vncClosed" });
    if (isVncWindow) void WebviewWindow.getCurrent().close();
  };
  const closeSsh = async () => {
    if (!isTerminalWindow) return;
    await closeTerminalSession(current().id);
  };
  const submitVncCredentials = async (event: SubmitEvent) => {
    event.preventDefault();
    if (!vncRfb) return;
    const enteredPassword = vncPassword();
    if (vncSavePassword() && selectedId()) {
      try {
        await savePassword(selectedId()!, enteredPassword);
        setPassword(enteredPassword);
      } catch (error) {
        setStatus({ key: "state.connectFailed", error });
        return;
      }
    }
    setVncCredentialsNeeded(false);
    vncRfb.sendCredentials({ password: enteredPassword });
    setVncPassword("");
  };
  const cancelVncCredentials = () => { setVncCredentialsNeeded(false); setVncPassword(""); vncRfb?.disconnect(); };
  createEffect(() => {
    const session = vncSession();
    if (!session || !vncDisplay) return;
    const savedPassword = password();
    const rfb = new RFB(vncDisplay, session.websocketUrl, {
      credentials: savedPassword ? { password: savedPassword } : {},
    });
    vncRfb = rfb;
    rfb.scaleViewport = true;
    rfb.resizeSession = false;
    rfb.addEventListener("credentialsrequired", () => {
      if (password()) {
        rfb.sendCredentials({ password: password() });
      } else {
        setVncPassword("");
        setVncSavePassword(false);
        setVncCredentialsNeeded(true);
      }
    });
    rfb.addEventListener("disconnect", (event: { detail?: { clean?: boolean } }) => {
      if (vncRfb === rfb) { vncRfb = undefined; setVncCredentialsNeeded(false); setVncSession(null); if (!event.detail?.clean) setStatus({ key: "state.connectFailed", error: "err.vncDisconnected" }); }
    });
    onCleanup(() => { if (vncRfb === rfb) rfb.disconnect(); });
  });

  /** Öffnet die VNC-Sitzung in einem eigenen Fenster. Ein Profil besitzt nur
   *  ein solches Fenster; ein zweiter Aufruf bringt das vorhandene nach vorn. */
  const openVncWindow = async (profile: RemoteProfile) => {
    const label = `vnc-${profile.id.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) { await existing.setFocus(); return; }
    const child = new WebviewWindow(label, {
      url: `/?vnc=${encodeURIComponent(profile.id)}`,
      title: `${profile.name} — RemoteDeskRDP`,
      width: 1280,
      height: 820,
      minWidth: 700,
      minHeight: 500,
      resizable: true,
      center: true,
    });
    await child.once("tauri://error", (event) => setStatus({ key: "state.connectFailed", error: String(event.payload) }));
  };
  const openTerminalWindow = async (profile: RemoteProfile) => {
    const label = `${profile.protocol}-${profile.id.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) { await existing.setFocus(); return; }
    new WebviewWindow(label, { url: `/?terminal=${encodeURIComponent(profile.id)}`, title: `${profile.name} — RemoteDeskRDP`, width: 1280, height: 820, minWidth: 700, minHeight: 500, resizable: true, center: true });
  };

  /** Verbindet ein Profil, das von aussen angefordert wurde (Deep-Link aus
   *  DualBeam). Unbekannte Kennungen werden gemeldet statt still verworfen.
   *  Meldet zurück, ob die Sitzung wirklich anlief – nur dann darf sich das
   *  Fenster wegklappen, sonst verschwände die Fehlermeldung mit ihm. */
  const openById = async (id: string): Promise<RemoteProfile | null> => {
    const profile = profiles().find((item) => item.id === id);
    if (!profile) { setStatus({ key: "state.linkUnknown" }); return null; }
    setCurrent(profile); setSelectedId(profile.id); setDirty(false);
    await loadSecret(profile.id);
    setStatus({ key: "state.connecting" });
    try {
      if (profile.protocol === "vnc") {
        await openVncWindow(profile);
        setStatus({ key: "state.vncWindowOpened", name: profile.name });
      } else if (["ssh", "sftp", "mosh"].includes(profile.protocol)) {
        await openTerminalWindow(profile);
        setStatus({ key: "state.connected", name: profile.name });
      } else {
        await connectProfile(profile);
        setStatus({ key: "state.connected", name: profile.name });
      }
      return profile;
    } catch (error) { setStatus({ key: "state.connectFailed", error }); return null; }
  };

  onMount(async () => {
    try {
      const loaded = await listProfiles();
      setProfiles(loaded);
      if (loaded[0]) { setCurrent(loaded[0]); setSelectedId(loaded[0].id); void loadSecret(loaded[0].id); }
    } catch (error) { setStatus({ key: "state.loadFailed", error }); }

    if (terminalProfileId) {
      const profile = profiles().find((item) => item.id === terminalProfileId);
      if (!profile) { setStatus({ key: "state.linkUnknown" }); return; }
      setCurrent(profile); setSelectedId(profile.id); setDirty(false); return;
    }
    if (vncProfileId) {
      const profile = profiles().find((item) => item.id === vncProfileId);
      if (!profile) { setStatus({ key: "state.linkUnknown" }); return; }
      setCurrent(profile); setSelectedId(profile.id); setDirty(false);
      await loadSecret(profile.id);
      setStatus({ key: "state.connecting" });
      try {
        await startVnc(profile);
        setStatus({ key: "state.vncConnected", name: profile.name });
      } catch (error) { setStatus({ key: "state.connectFailed", error }); }
      return;
    }

    // Das Ereignis ist nur ein Weckruf; die ID kommt immer aus dem Speicher.
    // Erst lauschen, dann abholen - sonst faellt eine URL, die genau dazwischen
    // eintrifft, durch beide Raster.
    const drainLink = async () => {
      let pending: string | null = null;
      try { pending = await takePendingLink(); }
      catch { return; /* ohne wartenden Link ist nichts zu tun */ }
      if (!pending) return;
      // RDP und VNC laufen jeweils in einem eigenen Fenster. Nach einem
      // DualBeam-Aufruf kann die Profilverwaltung daher stets wegklappen.
      const profile = await openById(pending);
      if (profile) await minimizeWindow();
    };
    await listen("deep-link-connect", () => void drainLink());
    await drainLink();
  });

  const update = <K extends Field>(field: K, value: RemoteProfile[K]) => {
    setCurrent((profile) => ({ ...profile, [field]: value, updatedAt: new Date().toISOString() }));
    setDirty(true);
  };
  const setFolders = (folders: SharedFolder[]) => update("sharedFolders", folders);
  /** FreeRDP trennt Name und Pfad am Komma, deshalb darf keines im Namen landen. */
  const addFolder = async () => {
    const chosen = await open({ directory: true, multiple: false, title: t("session.pickFolder") });
    if (typeof chosen !== "string") return;
    const fallback = t("session.shareFallbackName");
    const base = chosen.split("/").filter(Boolean).pop() ?? fallback;
    const folders = current().sharedFolders;
    if (folders.some((folder) => folder.path === chosen)) { setStatus({ key: "state.folderTwice" }); return; }
    let name = base.replace(/,/g, " ").trim() || fallback;
    while (folders.some((folder) => folder.name === name)) name = `${name}_`;
    setFolders([...folders, { name, path: chosen }]);
  };
  const renameFolder = (index: number, name: string) =>
    setFolders(current().sharedFolders.map((folder, position) => (position === index ? { ...folder, name } : folder)));
  const removeFolder = (index: number) =>
    setFolders(current().sharedFolders.filter((_, position) => position !== index));
  const selectProfile = async (profile: RemoteProfile) => {
    setCurrent(profile); setSelectedId(profile.id); setDirty(false);
    setStatus({ key: "state.profileLoaded" }); await loadSecret(profile.id);
  };
  const newProfile = () => {
    secretLoadGeneration++;
    const profile = emptyProfile();
    setCurrent(profile); setSelectedId(null); setPassword(""); setGatewayPassword("");
    setDirty(true); setStatus({ key: "state.newProfile" });
  };
  const clearPassword = async () => {
    const id = selectedId();
    if (!id) { setPassword(""); return; }
    try { await forgetPassword(id); setPassword(""); setStatus({ key: "state.passwordCleared" }); }
    catch (error) { setStatus({ key: "state.passwordClearFailed", error }); }
  };
  const clearGatewayPassword = async () => {
    const id = selectedId();
    if (!id) { setGatewayPassword(""); return; }
    try { await forgetGatewayPassword(id); setGatewayPassword(""); setStatus({ key: "state.gatewayPasswordCleared" }); }
    catch (error) { setStatus({ key: "state.gatewayPasswordClearFailed", error }); }
  };
  const persist = async () => {
    const profile = current();
    // Fehlercode statt Satz – die Oberfläche übersetzt ihn wie die des Backends.
    if (!profile.name.trim() || !profile.host.trim()) throw new Error("err.nameAndHostRequired");
    const saved = await saveProfile(profile);
    if (password()) await savePassword(saved.id, password());
    if (gatewayPassword()) await saveGatewayPassword(saved.id, gatewayPassword());
    setProfiles((items) => [saved, ...items.filter((item) => item.id !== saved.id)]);
    setCurrent(saved); setSelectedId(saved.id); setDirty(false);
    return saved;
  };
  const save = async () => {
    try { await persist(); setStatus({ key: "state.saved" }); }
    catch (error) { setStatus({ key: "state.saveFailed", error }); }
  };
  const remove = async () => {
    const id = selectedId();
    if (!id || !confirm(t("confirm.delete"))) return;
    try {
      await deleteProfile(id); setProfiles((items) => items.filter((item) => item.id !== id));
      newProfile(); setStatus({ key: "state.deleted" });
    } catch (error) { setStatus({ key: "state.deleteFailed", error }); }
  };
  const connect = async () => {
    setStatus({ key: "state.connecting" });
    try {
      const profile = await persist();
      if (profile.protocol === "vnc") {
        await openVncWindow(profile);
        setStatus({ key: "state.vncWindowOpened", name: profile.name });
      } else if (["ssh", "sftp", "mosh"].includes(profile.protocol)) {
        await openTerminalWindow(profile);
        setStatus({ key: "state.connected", name: profile.name });
      } else {
        await connectProfile(profile);
        setStatus({ key: "state.connected", name: profile.name });
      }
    }
    catch (error) { setStatus({ key: "state.connectFailed", error }); }
  };
  createEffect(() => { document.title = dirty() ? "• RemoteDeskRDP" : "RemoteDeskRDP"; });

  const themeTitle = () => t("side.themeTitle", { label: t(`theme.${getThemeMode()}`) });
  const langTitle = () => t("side.langTitle", { label: t(`lang.${getLangMode()}`) });

  return <main class="app-shell" classList={{ "vnc-window": isVncWindow, "ssh-window": isTerminalWindow }}>
    <aside class="sidebar">
      <div class="brand"><div class="brand-mark">◫</div><div><strong>RemoteDeskRDP</strong><span>{t("brand.subtitle")}</span></div></div>
      <div class="side-label">{t("side.connections")}</div>
      <div class="profile-list">
        <For each={profiles()}>{(profile) => <button classList={{ profile: true, selected: profile.id === selectedId() }} onClick={() => selectProfile(profile)}>
          <span class="dot"></span><span class="profile-copy"><strong>{profile.name}</strong><small>{profile.protocol.toUpperCase()} · {profile.host || t("side.notConfigured")}</small></span>
        </button>}</For>
        <Show when={profiles().length === 0}><div class="empty-list">{t("side.empty")}</div></Show>
      </div>
      <div class="side-footer">
        <button onClick={newProfile}>{t("side.newProfile")}</button>
        <button class="quiet" onClick={() => setHelpOpen(true)}>{t("side.help")}</button>
        <button class="quiet" onClick={() => setLicenseOpen(true)}>{t("side.license")}</button>
        <div class="side-switches">
          <button class="quiet switch-btn" title={themeTitle()} aria-label={themeTitle()} onClick={cycleThemeMode}>{themeIcon(getThemeMode())}</button>
          <button class="quiet switch-btn" title={langTitle()} aria-label={langTitle()} onClick={cycleLangMode}>{langIcon(getLangMode())}</button>
        </div>
      </div>
    </aside>

    <section class="editor">
      <header class="topbar"><div><span class="eyebrow">{t("top.eyebrow")}</span><h1>{current().name || t("top.untitled")}</h1></div><div class="status-chip"><span></span>{statusText()}</div></header>
      <Show when={isTerminalWindow && ["ssh", "sftp", "mosh"].includes(current().protocol)}><section class="ssh-session"><header><div><span class="eyebrow">{current().protocol.toUpperCase()}</span><h2>{current().name}</h2></div><button class="secondary" onClick={() => void closeSsh()}>{t("ssh.disconnect")}</button></header><div class="ssh-frame"><SshTerminal profile={current()} /></div></section></Show>
      <Show when={vncSession()}>{(session) => <section class="vnc-session"><header><div><span class="eyebrow">VNC</span><h2>{session().profile.name}</h2></div><button class="secondary" onClick={closeVnc}>{t("vnc.disconnect")}</button></header><div class="vnc-frame"><div class="vnc-display" ref={(element) => { vncDisplay = element; }} /><Show when={vncCredentialsNeeded()}><form class="vnc-auth" onSubmit={submitVncCredentials}><h3>{t("vnc.passwordRequired")}</h3><p>{t("vnc.passwordHelp", { name: session().profile.name })}</p><input type="password" value={vncPassword()} onInput={(event) => setVncPassword(event.currentTarget.value)} autofocus /><label class="vnc-save-password"><input type="checkbox" checked={vncSavePassword()} onChange={(event) => setVncSavePassword(event.currentTarget.checked)} />{t("vnc.savePassword")}</label><div><button type="button" class="secondary" onClick={cancelVncCredentials}>{t("vnc.cancel")}</button><button class="connect" type="submit">{t("action.connect")}</button></div></form></Show></div></section>}</Show>
      <div class="editor-scroll" classList={{ hidden: !!vncSession() || isVncWindow || isTerminalWindow }}>
        <section class="identity-card">
          <div class="field-grid">
            <label class="wide"><span>{t("field.name")}</span><input value={current().name} onInput={(event) => update("name", event.currentTarget.value)} placeholder={t("field.namePlaceholder")} /></label>
            <label><span>{t("field.protocol")}</span><select value={current().protocol} onChange={(event) => update("protocol", event.currentTarget.value as Protocol)}><option value="rdp">{t("protocol.rdp")}</option><option value="vnc">{t("protocol.vnc")}</option><option value="ssh">{t("protocol.ssh")}</option><option value="sftp">{t("protocol.sftp")}</option><option value="mosh">{t("protocol.mosh")}</option></select></label>
            <label class="wide"><span>{t("field.host")}</span><input value={current().host} onInput={(event) => update("host", event.currentTarget.value)} placeholder={t("field.hostPlaceholder")} /></label>
            <Show when={current().protocol !== "vnc"}><label><span>{t("field.username")}</span><input value={current().username} onInput={(event) => update("username", event.currentTarget.value)} placeholder={t("field.optional")} /></label>
              <Show when={current().protocol === "rdp"}>
              <label><span>{t("field.domain")}</span><input value={current().domain} onInput={(event) => update("domain", event.currentTarget.value)} placeholder={t("field.optional")} /></label>
              <label class="wide"><span>{t("field.password")}</span><div class="password-input"><input type={revealPassword() ? "text" : "password"} value={password()} onInput={(event) => setPassword(event.currentTarget.value)} placeholder={t("field.passwordPlaceholder")} /><button type="button" onClick={() => setRevealPassword((value) => !value)}>{revealPassword() ? t("action.hide") : t("action.reveal")}</button><Show when={password()}><button type="button" onClick={() => void clearPassword()}>{t("action.remove")}</button></Show></div><small>{t("field.passwordNote")}</small></label></Show>
              </Show>
          </div>
        </section>

        <section class="section-card"><div class="section-head"><div><span class="eyebrow">{t("transport.eyebrow")}</span><h2>{t("transport.title")}</h2></div><span class="secure-note">{t("transport.fallback")}</span></div>
          <Show when={current().protocol === "rdp"} fallback={<Show when={current().protocol === "vnc"} fallback={<><div class="port-row"><Port label={t("terminal.port")} value={current().sshPort} onChange={(value) => update("sshPort", value)} /><label class="port"><span>{t("ssh.terminal")}</span><select value={current().sshTerminal} onChange={(event) => update("sshTerminal", event.currentTarget.value)}><option value="xterm-256color">{t("ssh.terminal.xterm256")}</option><option value="xterm">{t("ssh.terminal.xterm")}</option><option value="screen-256color">{t("ssh.terminal.screen")}</option><option value="tmux-256color">{t("ssh.terminal.tmux")}</option><option value="vt100">{t("ssh.terminal.vt100")}</option><option value="linux">{t("ssh.terminal.linux")}</option></select></label></div><p>{current().protocol === "sftp" ? t("sftp.note") : current().protocol === "mosh" ? t("mosh.note") : t("ssh.terminalNote")}</p></>}><div class="port-row"><Port label="VNC" value={current().vncPort} onChange={(value) => update("vncPort", value)} /></div><p>{t("vnc.note")}</p></Show>}><div class="port-row"><Port label="RDP TCP" value={current().rdpTcpPort} onChange={(value) => update("rdpTcpPort", value)} /><Port label="RDP UDP" value={current().rdpUdpPort} onChange={(value) => update("rdpUdpPort", value)} /><Toggle label={t("transport.udpPreferred")} checked={current().udpPreferred} onChange={(value) => update("udpPreferred", value)} /><Toggle label={t("transport.autoReconnect")} checked={current().autoReconnect} onChange={(value) => update("autoReconnect", value)} /></div><p>{t("transport.note")}</p></Show>
          <Show when={current().protocol === "ssh"}>
            <div class="x11-settings">
              <Toggle label={t("ssh.x11.enable")} checked={current().x11Forwarding} onChange={(value) => update("x11Forwarding", value)} />
              <Show when={current().x11Forwarding}>
                <label class="x11-command"><span>{t("ssh.x11.command")}</span><input value={current().x11Command} onInput={(event) => update("x11Command", event.currentTarget.value)} placeholder={t("ssh.x11.commandPlaceholder")} /><small>{t("ssh.x11.commandNote")}</small></label>
                <Toggle label={t("ssh.x11.closeXquartz")} checked={current().x11CloseXquartz} onChange={(value) => update("x11CloseXquartz", value)} />
                <p>{t("ssh.x11.note")}</p>
              </Show>
            </div>
          </Show>
        </section>

        <Show when={current().protocol === "rdp"}><section class="section-card"><div class="section-head"><div><span class="eyebrow">{t("gateway.eyebrow")}</span><h2>{t("gateway.title")}</h2></div></div>
          <Toggle label={t("gateway.enable")} checked={current().gatewayEnabled} onChange={(value) => update("gatewayEnabled", value)} />
          <Show when={current().gatewayEnabled}>
            <div class="field-grid">
              <label class="wide"><span>{t("gateway.host")}</span><input value={current().gatewayHost} onInput={(event) => update("gatewayHost", event.currentTarget.value)} placeholder="gateway.example.net" /><small>{t("gateway.hostNote")}</small></label>
              <label><span>{t("gateway.port")}</span><input type="number" min="1" max="65535" value={current().gatewayPort} onInput={(event) => update("gatewayPort", Number(event.currentTarget.value) || 443)} /></label>
              <label><span>{t("gateway.username")}</span><input value={current().gatewayUsername} onInput={(event) => update("gatewayUsername", event.currentTarget.value)} placeholder={t("gateway.sameAsSession")} /></label>
              <label><span>{t("gateway.domain")}</span><input value={current().gatewayDomain} onInput={(event) => update("gatewayDomain", event.currentTarget.value)} placeholder={t("gateway.sameAsSession")} /></label>
              <label class="wide"><span>{t("gateway.password")}</span><div class="password-input"><input type={revealGatewayPassword() ? "text" : "password"} value={gatewayPassword()} onInput={(event) => setGatewayPassword(event.currentTarget.value)} placeholder={t("gateway.sameAsSession")} /><button type="button" onClick={() => setRevealGatewayPassword((value) => !value)}>{revealGatewayPassword() ? t("action.hide") : t("action.reveal")}</button><Show when={gatewayPassword()}><button type="button" onClick={() => void clearGatewayPassword()}>{t("action.remove")}</button></Show></div><small>{t("gateway.passwordNote")}</small></label>
            </div>
            <p>{t("gateway.note")}</p>
          </Show>
        </section></Show>

        <Show when={current().protocol === "rdp"}><section class="section-card"><div class="section-head"><div><span class="eyebrow">{t("display.eyebrow")}</span><h2>{t("display.title")}</h2></div></div>
          <label class="select-label"><span>{t("display.mode")}</span><select value={current().displayMode} onChange={(event) => update("displayMode", event.currentTarget.value as DisplayMode)}>
            <option value="window">{t("display.mode.window")}</option>
            <option value="workarea">{t("display.mode.workarea")}</option>
            <option value="fullscreen">{t("display.mode.fullscreen")}</option>
          </select></label>
          <Show when={current().displayMode === "window"}>
            <div class="port-row">
              <Pixels label={t("display.width")} value={current().width} onChange={(value) => update("width", value)} />
              <Pixels label={t("display.height")} value={current().height} onChange={(value) => update("height", value)} />
            </div>
            <div class="preset-row"><For each={resolutionPresets}>{(preset) => <button type="button" classList={{ preset: true, active: current().width === preset.width && current().height === preset.height }} onClick={() => { update("width", preset.width); update("height", preset.height); }}>{preset.label}</button>}</For></div>
          </Show>
          <label class="select-label"><span>{t("display.resize")}</span><select value={current().resizeBehavior} onChange={(event) => update("resizeBehavior", event.currentTarget.value as ResizeBehavior)}>
            <option value="dynamic">{t("display.resize.dynamic")}</option>
            <option value="scale">{t("display.resize.scale")}</option>
            <option value="fixed">{t("display.resize.fixed")}</option>
          </select></label>
          <label class="select-label"><span>{t("display.depth")}</span><select value={current().colorDepth} onChange={(event) => update("colorDepth", event.currentTarget.value as ColorDepth)}>
            <option value="auto">{t("display.depth.auto")}</option>
            <option value="32">{t("display.depth.32")}</option>
            <option value="24">{t("display.depth.24")}</option>
            <option value="16">{t("display.depth.16")}</option>
            <option value="15">{t("display.depth.15")}</option>
            <option value="8">{t("display.depth.8")}</option>
          </select></label>
          <p>{t("display.depthNote")}</p>
          <p>{t("display.resizeNote")}</p>
        </section></Show>

        <Show when={current().protocol === "rdp"}><section class="section-card"><div class="section-head"><div><span class="eyebrow">{t("session.eyebrow")}</span><h2>{t("session.title")}</h2></div><span class="keychain">{t("session.keychain")}</span></div>
          <div class="toggle-grid"><Toggle label={t("session.clipboard")} checked={current().clipboard} onChange={(value) => update("clipboard", value)} /><Toggle label={t("session.audio")} checked={current().audio} onChange={(value) => update("audio", value)} /><Toggle label={t("session.printer")} checked={current().printer} onChange={(value) => update("printer", value)} /><Toggle label={t("session.smartcard")} checked={current().smartcard} onChange={(value) => update("smartcard", value)} /><Toggle label={t("session.video")} checked={current().video} onChange={(value) => update("video", value)} /></div>
          <div class="shares">
            <div class="shares-head"><span>{t("session.shares")}</span><button class="secondary small" onClick={addFolder}>{t("session.addFolder")}</button></div>
            <Show when={current().sharedFolders.length > 0} fallback={<p class="shares-empty">{t("session.noShares")}</p>}>
              <ul class="share-list"><For each={current().sharedFolders}>{(folder, index) => (
                <li>
                  <input value={folder.name} onInput={(event) => renameFolder(index(), event.currentTarget.value)} aria-label={t("session.shareName")} />
                  <span title={folder.path}>{folder.path}</span>
                  <button class="danger-quiet" onClick={() => removeFolder(index())} aria-label={t("session.removeShare")}>{t("session.remove")}</button>
                </li>
              )}</For></ul>
            </Show>
          </div>
          <label class="select-label"><span>{t("session.certificates")}</span><select value={current().certificateMode} onChange={(event) => update("certificateMode", event.currentTarget.value as RemoteProfile["certificateMode"])}><option value="prompt">{t("session.cert.prompt")}</option><option value="tofu">{t("session.cert.tofu")}</option><option value="ignore">{t("session.cert.ignore")}</option></select></label>
        </section></Show>
      </div>
      <Show when={!vncSession() && !isVncWindow && !isTerminalWindow}><footer class="actionbar"><button type="button" class="danger-quiet" disabled={!selectedId()} onClick={remove}>{t("action.deleteProfile")}</button><div class="actions"><button type="button" class="secondary" onClick={save}>{t("action.saveProfile")}</button><button type="button" class="connect" onClick={connect}>{t("action.connect")} <span>→</span></button></div></footer></Show>
    </section>

    <Show when={helpOpen()}>
      <div class="help-sheet">
        <div class="help-panel">
          <header><div><span class="eyebrow">{t("help.eyebrow")}</span><h2>{t("help.title")}</h2></div><button onClick={() => setHelpOpen(false)}>{t("help.close")}</button></header>
          <div class="help-content">
            {/* Die Abschnitte enthalten Auszeichnungen wie <b> und <ul>. Der Inhalt
                stammt ausschliesslich aus den Woerterbuechern, nie aus Eingaben. */}
            <For each={helpSections}>{(section) => <>
              <h3>{t(`help.${section}.h`)}</h3>
              <div class="help-body" innerHTML={t(`help.${section}.body`)} />
            </>}</For>
          </div>
        </div>
      </div>
    </Show>

    <Show when={licenseOpen()}>
      <div class="help-sheet">
        <div class="help-panel">
          <header><div><span class="eyebrow">{t("license.eyebrow")}</span><h2>{t("license.title")}</h2></div><button onClick={() => setLicenseOpen(false)}>{t("license.close")}</button></header>
          <div class="help-content">
            {/* Wie bei der Hilfe: die Auszeichnungen stammen ausschliesslich
                aus den Woerterbuechern, nie aus Eingaben. */}
            <For each={licenseSections}>{(section) => <>
              <h3>{t(`license.${section}.h`)}</h3>
              <div class="help-body" innerHTML={t(`license.${section}.body`)} />
            </>}</For>
          </div>
        </div>
      </div>
    </Show>
  </main>;
}

function Port(props: { label: string; value: number; onChange: (value: number) => void }) {
  return <label class="port"><span>{props.label}</span><div><input type="number" min="1" max="65535" value={props.value} onChange={(event) => props.onChange(readPort(event.currentTarget.value, props.value))} /><b>{t("unit.port")}</b></div></label>;
}
function Pixels(props: { label: string; value: number; onChange: (value: number) => void }) {
  return <label class="port"><span>{props.label}</span><div><input type="number" min="640" max="7680" step="2" value={props.value} onChange={(event) => props.onChange(readPixels(event.currentTarget.value, props.value))} /><b>{t("unit.px")}</b></div></label>;
}
function Toggle(props: { label: string; checked: boolean; onChange: (value: boolean) => void }) {
  return <label class="toggle"><input type="checkbox" checked={props.checked} onChange={(event) => props.onChange(event.currentTarget.checked)} /><span class="switch"><i></i></span><span>{props.label}</span></label>;
}
