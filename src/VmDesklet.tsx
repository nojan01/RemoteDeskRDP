import { For, Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { availableMonitors, getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize, PhysicalPosition } from "@tauri-apps/api/dpi";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { createDeskletShortcut, pingVm, validateVmAddress } from "./api";
import { newVmTarget, parseDeskletSettings, type DeskletSettings, type VmTarget } from "./deskletSettings";
import { parseDeskletPosition, visibleDeskletPosition } from "./deskletPosition";
import { errMsg, t } from "./i18n";
import "./desklet.css";

const settingsKey = "remotedesk.vmDesklet";
type Status = "idle" | "checking" | "online" | "offline" | "error";
type VmState = { status: Status; error?: string; lastChecked?: Date; busy?: boolean };

function readSettings(): DeskletSettings {
  try { return parseDeskletSettings(JSON.parse(localStorage.getItem(settingsKey) ?? "null")); }
  catch { return parseDeskletSettings(null); }
}

export function VmDesklet() {
  const initial = readSettings();
  const [editing, setEditing] = createSignal(initial.vms.length === 0);
  const [drafts, setDrafts] = createSignal<VmTarget[]>(initial.vms.length ? initial.vms.map(vm => ({ ...vm })) : [newVmTarget()]);
  const [opacity, setOpacity] = createSignal(initial.opacity);
  const [active, setActive] = createSignal<DeskletSettings>(initial);
  const [states, setStates] = createSignal<Record<string, VmState>>({});
  const [formError, setFormError] = createSignal("");
  const [invalidVm, setInvalidVm] = createSignal(0);
  const [saving, setSaving] = createSignal(false);
  const [shortcutBusy, setShortcutBusy] = createSignal(false);
  const [shortcutPath, setShortcutPath] = createSignal("");
  const [shortcutError, setShortcutError] = createSignal("");
  let generation = 0;
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  const pending = new Map<string, Promise<boolean>>();
  const deskletWindow = getCurrentWindow();
  const positionKey = `remotedesk.deskletPosition:${deskletWindow.label}`;
  let unlistenMove: UnlistenFn | undefined;
  let layoutQueue = Promise.resolve();
  let disposed = false;
  const state = (id: string): VmState => states()[id] ?? { status: "idle" };
  const patchState = (id: string, patch: Partial<VmState>) => setStates(previous => ({ ...previous, [id]: { ...previous[id], ...patch } }));

  function stopTimers() {
    for (const timer of timers.values()) clearTimeout(timer);
    timers.clear();
  }

  // Jede VM hat einen eigenen Zeitplan. Eine nicht erreichbare VM verzögert
  // keine andere; Ergebnisse einer alten Konfiguration werden verworfen.
  async function check(vm: VmTarget, version: number) {
    await pending.get(vm.id)?.catch(() => {});
    if (version !== generation) return;
    patchState(vm.id, { busy: true });
    const started = Date.now();
    const request = pingVm(vm.address);
    pending.set(vm.id, request);
    try {
      const reachable = await request;
      if (version !== generation) return;
      patchState(vm.id, { status: reachable ? "online" : "offline", error: undefined });
    } catch (reason) {
      if (version !== generation) return;
      patchState(vm.id, { status: "error", error: String(reason) === "desklet.invalidIp" ? "desklet.invalidIp" : "desklet.pingError" });
    } finally {
      if (pending.get(vm.id) === request) pending.delete(vm.id);
      if (version === generation) {
        patchState(vm.id, { busy: false, lastChecked: new Date() });
        timers.set(vm.id, setTimeout(() => void check(vm, version), Math.max(0, vm.interval * 1000 - (Date.now() - started))));
      }
    }
  }

  function start(settings: DeskletSettings) {
    stopTimers();
    generation += 1;
    setActive(settings);
    setStates(Object.fromEntries(settings.vms.map(vm => [vm.id, { status: "checking" }])));
    for (const vm of settings.vms) void check(vm, generation);
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (saving()) return;
    const settings = { vms: drafts().map(vm => ({ ...vm, name: vm.name.trim(), address: vm.address.trim() })), opacity: opacity() };
    if (!settings.vms.length) return;
    setFormError("");
    setInvalidVm(0);
    for (const [index, vm] of settings.vms.entries()) {
      if (!Number.isInteger(vm.interval) || vm.interval < 3 || vm.interval > 3600 || !vm.address) {
        setFormError(vm.address ? "desklet.invalidInterval" : "desklet.invalidIp");
        setInvalidVm(index + 1);
        return;
      }
    }
    setSaving(true);
    try {
      const validated = await Promise.all(settings.vms.map(async (vm, index) => {
        try { return { ...vm, address: await validateVmAddress(vm.address) }; }
        catch { throw index + 1; }
      }));
      settings.vms = validated;
    } catch (index) {
      setFormError("desklet.invalidIp");
      setInvalidVm(Number(index));
      setSaving(false);
      return;
    }
    try { localStorage.setItem(settingsKey, JSON.stringify(settings)); }
    catch { setFormError("desklet.saveError"); setSaving(false); return; }
    setSaving(false);
    setDrafts(settings.vms.map(vm => ({ ...vm })));
    start(settings);
    setEditing(false);
  }

  function toggleSettings() {
    if (!editing()) {
      setDrafts(active().vms.map(vm => ({ ...vm })));
      setOpacity(active().opacity);
      setFormError("");
      setInvalidVm(0);
    }
    setEditing(!editing());
  }

  createEffect(() => {
    const count = active().vms.length;
    const height = count <= 1 ? 230 : 48 + count * 132;
    const availableHeight = Math.max(230, window.screen.availHeight - 80);
    const size = editing() ? new LogicalSize(370, Math.min(700, availableHeight)) : new LogicalSize(310, Math.min(height, availableHeight));
    // Erst Größe und gespeicherte Position herstellen, dann sichtbar machen.
    // Die Warteschlange verhindert überholte Layout-Aufrufe beim Speichern.
    layoutQueue = layoutQueue.then(async () => {
      if (disposed) return;
      await deskletWindow.setSize(size);
      if (!unlistenMove && !disposed) {
        let saved = null;
        try { saved = parseDeskletPosition(JSON.parse(localStorage.getItem(positionKey) ?? "null")); }
        catch { /* Fehlende/beschädigte Positionsdaten verwenden den Standard. */ }
        if (saved) {
          const [monitors, actualSize] = await Promise.all([availableMonitors(), deskletWindow.outerSize()]);
          const position = visibleDeskletPosition(saved, actualSize, monitors.map(monitor => ({
            x: monitor.workArea.position.x, y: monitor.workArea.position.y,
            width: monitor.workArea.size.width, height: monitor.workArea.size.height,
          })));
          await deskletWindow.setPosition(new PhysicalPosition(position.x, position.y));
        }
        const unlisten = await deskletWindow.onMoved(({ payload }) => {
          try { localStorage.setItem(positionKey, JSON.stringify({ x: payload.x, y: payload.y })); }
          catch (error) { console.error("Desklet position could not be saved", error); }
        });
        if (disposed) unlisten();
        else unlistenMove = unlisten;
      }
    }).catch(error => console.error("Desklet layout failed", error)).then(async () => {
      if (!disposed) await deskletWindow.show();
    }).catch(error => console.error("Desklet could not be shown", error));
  });

  async function createShortcut() {
    if (shortcutBusy()) return;
    setShortcutBusy(true);
    setShortcutError("");
    setShortcutPath("");
    try { setShortcutPath(await createDeskletShortcut()); }
    catch (error) { setShortcutError(errMsg(error)); }
    finally { setShortcutBusy(false); }
  }
  onMount(() => {
    document.documentElement.classList.add("vm-desklet-page");
    if (initial.vms.length) start(initial);
  });
  onCleanup(() => {
    disposed = true;
    unlistenMove?.();
    generation += 1;
    stopTimers();
    document.documentElement.classList.remove("vm-desklet-page");
  });

  const headerLabel = () => active().vms.length === 1
    ? t("desklet.every", { seconds: active().vms[0].interval })
    : t("desklet.vmCount", { count: active().vms.length });

  return <main class="vm-desklet" classList={{ "vm-compact": !editing(), "vm-multiple": active().vms.length > 1 }}
    style={{ "background-color": `color-mix(in srgb, var(--bg) ${editing() ? 100 : active().opacity}%, transparent)` }}>
    <header onMouseDown={event => {
      if (event.button === 0 && !(event.target as HTMLElement).closest("button")) {
        void getCurrentWindow().startDragging().catch(error => console.error("Desklet drag failed", error));
      }
    }}><strong>VM Ping</strong><div class="vm-header-actions"><span>{headerLabel()}</span>
      <Show when={active().vms.length}><button class="vm-settings-button" type="button" onClick={toggleSettings} aria-label={t(editing() ? "desklet.closeSettings" : "desklet.settings")} title={t(editing() ? "desklet.closeSettings" : "desklet.settings")} aria-expanded={editing()}>{editing() ? "×" : "⚙"}</button></Show>
      <button class="vm-settings-button vm-power-button" type="button" onClick={() => void getCurrentWindow().close().catch(error => console.error("Desklet close failed", error))} aria-label={t("desklet.powerOff")} title={t("desklet.powerOff")}>⏻</button>
    </div></header>
    <Show when={!editing()} fallback={<Show when={!active().vms.length}><p class="vm-note">{t("desklet.enterIp")}</p></Show>}>
      <For each={active().vms}>{vm => <article class="vm-monitor" aria-label={vm.name || vm.address}>
        <section class={`vm-status vm-${state(vm.id).status}`} role="status" aria-live="polite" title={state(vm.id).error ? t(state(vm.id).error!) : undefined}>
          <span class="vm-status-dot" aria-hidden="true" />
          <div class="vm-status-copy">
            <strong>{t(`desklet.${state(vm.id).status}`)}</strong>
            <Show when={vm.name}><span class="vm-name" title={vm.name}>{vm.name}</span></Show>
            <span class="vm-address">{vm.address}</span>
          </div>
        </section>
        <div class="vm-last-check" aria-busy={state(vm.id).busy}>
          {state(vm.id).lastChecked ? t("desklet.lastCheck", { time: state(vm.id).lastChecked!.toLocaleTimeString() }) : t("desklet.checking")}
          <Show when={active().vms.length > 1}><span class="vm-target-interval"> · {t("desklet.every", { seconds: vm.interval })}</span></Show>
        </div>
      </article>}</For>
    </Show>
    <Show when={editing()}>
      <form class="vm-setup-form" onSubmit={save}>
        <For each={drafts()}>{(vm, index) => <fieldset class="vm-target-fields" disabled={saving()}>
          <legend>{t("desklet.vmNumber", { number: index() + 1 })}</legend>
          <label>{t("desklet.name")}<input value={vm.name} onInput={event => { vm.name = event.currentTarget.value; }} placeholder={t("desklet.namePlaceholder")} maxlength="80" /></label>
          <label>{t("desklet.ip")}<input value={vm.address} onInput={event => { vm.address = event.currentTarget.value; }} placeholder="192.168.1.20" required spellcheck={false} autocomplete="off" /></label>
          <label>{t("desklet.interval")}<input type="number" min="3" max="3600" step="1" required value={vm.interval} onInput={event => { vm.interval = Number(event.currentTarget.value); }} /></label>
          <Show when={drafts().length > 1}><button class="vm-remove-button" type="button" onClick={() => setDrafts(previous => previous.filter(target => target.id !== vm.id))}>{t("desklet.removeVm")}</button></Show>
        </fieldset>}</For>
        <button class="vm-add-button" type="button" disabled={saving()} onClick={() => setDrafts(previous => [...previous, newVmTarget()])}>{t("desklet.addVm")}</button>
        <label>{t("desklet.transparency", { percent: 100 - opacity() })}<input class="vm-opacity-slider" type="range" min="0" max="100" step="5" value={100 - opacity()} onInput={event => setOpacity(100 - Number(event.currentTarget.value))} /></label>
        <p class="vm-note vm-opacity-note">{t("desklet.transparencyHint")}</p>
        <Show when={formError()}><p class="vm-error" role="alert"><Show when={invalidVm()}>{t("desklet.vmNumber", { number: invalidVm() })}: </Show>{t(formError())}</p></Show>
        <button type="submit" disabled={saving()}>{t("desklet.apply")}</button>
      </form>
      <button class="vm-shortcut-button" type="button" disabled={shortcutBusy()} onClick={() => void createShortcut()}>{t("desklet.createShortcut")}</button>
      <Show when={shortcutPath()}><p class="vm-note" role="status">{t("desklet.shortcutCreated", { path: shortcutPath() })}</p></Show>
      <Show when={shortcutError()}><p class="vm-error" role="alert">{shortcutError()}</p></Show>
      <p class="vm-note">{t("desklet.note")}</p>
    </Show>
  </main>;
}
