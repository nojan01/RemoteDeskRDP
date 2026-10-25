//! Optionaler Menüleisten-Betrieb und Start bei der Anmeldung.
//!
//! Beide Optionen sind abschaltbar und standardmäßig aus. Ohne sie verhält sich
//! die App wie bisher: Das Schließen des Fensters beendet sie.
//!
//! Mit Menüleisten-Betrieb bleibt die App nach dem Schließen des Fensters als
//! Symbol in der Menüleiste aktiv. Das Dock-Symbol verschwindet, solange kein
//! Fenster offen ist. Gespeicherte Verbindungen lassen sich direkt aus dem
//! Symbolmenü starten.

use crate::{config_dir, err1, read_profiles, Protocol};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Fenster beim Start: noch offen, gezeigt oder für einen Link zurückgehalten.
const LAUNCH_PENDING: u8 = 0;
const LAUNCH_SHOWN: u8 = 1;
const LAUNCH_WITHHELD: u8 = 2;
static LAUNCH_WINDOW: AtomicU8 = AtomicU8::new(LAUNCH_PENDING);
/// So lange wartet das Fenster beim Start auf einen Verbindungslink.
const LINK_GRACE: Duration = Duration::from_millis(400);

const SETTINGS_FILE: &str = "settings.json";
const TRAY_ID: &str = "remotedesk-tray";
const TRAY_CONNECT_PREFIX: &str = "tray:connect:";
const TRAY_OPEN_ID: &str = "tray:open";
const TRAY_QUIT_ID: &str = "tray:quit";
const TRAY_EMPTY_ID: &str = "tray:empty";
/// Kennzeichnet den Start durch den Anmeldeagenten. Nur dann, und nur mit
/// Menüleisten-Betrieb, startet die App ohne Fenster.
pub(crate) const BACKGROUND_ARGUMENT: &str = "--background";
/// Längere Profilnamen sprengen sonst die Breite des Menüs.
const MAX_MENU_NAME: usize = 60;

/// Einstellungen, wie sie die Oberfläche sieht.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    pub menu_bar: bool,
    pub launch_at_login: bool,
    pub desklet_at_startup: bool,
}

/// Inhalt von `settings.json`. Der Anmeldestart steht bewusst nicht darin:
/// Maßgeblich ist allein, ob der Anmeldeagent existiert. Sonst könnten Datei
/// und System auseinanderlaufen, etwa wenn der Nutzer den Eintrag in den
/// Systemeinstellungen entfernt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct StoredSettings {
    menu_bar: bool,
    desklet_at_startup: bool,
}

/// Menüleisten-Betrieb im Speicher; gelesen bei jedem Schließen des Fensters.
#[derive(Default)]
pub(crate) struct MenuBarState(AtomicBool);

/// Beschriftungen des Symbolmenüs. Wie beim Update-Menü kennt nur die
/// Oberfläche die gewählte Sprache.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrayLabels {
    open: String,
    quit: String,
    empty: String,
}

impl Default for TrayLabels {
    fn default() -> Self {
        Self {
            open: "Open RemoteDeskRDP".into(),
            quit: "Quit RemoteDeskRDP".into(),
            empty: "No connections".into(),
        }
    }
}

#[derive(Default)]
pub(crate) struct TrayLabelState(Mutex<TrayLabels>);

fn read_stored() -> StoredSettings {
    config_dir()
        .ok()
        .and_then(|directory| fs::read_to_string(directory.join(SETTINGS_FILE)).ok())
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

fn write_stored(settings: &StoredSettings) -> Result<(), String> {
    let directory = config_dir()?;
    fs::create_dir_all(&directory).map_err(|e| err1("err.settingsWrite", e))?;
    let data = serde_json::to_string_pretty(settings).map_err(|e| err1("err.settingsWrite", e))?;
    let target = directory.join(SETTINGS_FILE);
    let temporary = directory.join(format!("{SETTINGS_FILE}.{}.tmp", std::process::id()));
    fs::write(&temporary, data)
        .and_then(|_| fs::rename(&temporary, &target))
        .map_err(|e| {
            let _ = fs::remove_file(&temporary);
            err1("err.settingsWrite", e)
        })
}

/// Lädt die gespeicherte Einstellung beim Start.
pub(crate) fn load(app: &AppHandle) {
    app.state::<MenuBarState>().0.store(read_stored().menu_bar, Ordering::Relaxed);
}

pub(crate) fn menu_bar_enabled(app: &AppHandle) -> bool {
    app.state::<MenuBarState>().0.load(Ordering::Relaxed)
}

/// Ob die App ohne Fenster starten soll: nur beim Anmeldestart und nur, wenn
/// das Menüleisten-Symbol den Weg zurück ins Fenster bietet.
pub(crate) fn start_hidden(app: &AppHandle) -> bool {
    menu_bar_enabled(app) && std::env::args().any(|argument| argument == BACKGROUND_ARGUMENT)
}

/// Zeigt das Hauptfenster beim Start – außer, die App wurde für einen
/// Verbindungslink gestartet (Schreibtisch-Verknüpfung, DualBeam).
///
/// macOS reicht den Link erst nach `setup` nach, wenn die Ereignisschleife
/// läuft. Ohne kurze Wartezeit stünde das Fenster also schon da, bevor die
/// App vom Link erfährt. Ein Start ohne Link zeigt das Fenster danach wie gewohnt.
pub(crate) fn show_at_launch(app: &AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(LINK_GRACE);
        if LAUNCH_WINDOW
            .compare_exchange(LAUNCH_PENDING, LAUNCH_SHOWN, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            show_main_window(&handle);
        }
    });
}

/// Ein Verbindungslink ist eingetroffen. Kam er beim Start, bleibt das Fenster
/// verborgen; die Oberfläche holt es nur für Rückfragen und Fehler hervor.
///
/// Außerhalb von macOS bliebe ohne Menüleisten-Symbol kein Weg zurück ins
/// Fenster, denn dort fehlt das Dock.
pub(crate) fn withhold_for_link(app: &AppHandle) {
    if !(cfg!(target_os = "macos") || menu_bar_enabled(app)) {
        return;
    }
    let _ = LAUNCH_WINDOW.compare_exchange(LAUNCH_PENDING, LAUNCH_WITHHELD, Ordering::AcqRel, Ordering::Acquire);
}

/// Ob das Fenster seit dem Start für einen Link verborgen geblieben ist.
pub(crate) fn withheld() -> bool {
    LAUNCH_WINDOW.load(Ordering::Acquire) == LAUNCH_WITHHELD
}

#[tauri::command]
pub(crate) fn get_app_settings(app: AppHandle) -> AppSettings {
    AppSettings {
        menu_bar: menu_bar_enabled(&app),
        launch_at_login: login_item::enabled(),
        desklet_at_startup: desklet_at_startup_enabled(),
    }
}

pub(crate) fn desklet_at_startup_enabled() -> bool {
    read_stored().desklet_at_startup
}

/// Übernimmt geänderte Einstellungen sofort. Der Menüleisten-Betrieb wird vor
/// dem Anmeldestart gespeichert, damit ein Fehler beim Agenten die andere
/// Änderung nicht mitreißt; die Oberfläche liest danach den echten Stand neu.
#[tauri::command]
pub(crate) fn set_app_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let stored = StoredSettings {
        menu_bar: settings.menu_bar,
        desklet_at_startup: settings.desklet_at_startup,
    };
    if stored != read_stored() {
        write_stored(&stored)?;
    }
    if settings.menu_bar != menu_bar_enabled(&app) {
        app.state::<MenuBarState>().0.store(settings.menu_bar, Ordering::Relaxed);
        sync_tray(&app);
        refresh_dock(&app);
    }
    if settings.launch_at_login != login_item::enabled() {
        login_item::set(&app, settings.launch_at_login)?;
    }
    Ok(get_app_settings(app))
}

#[tauri::command]
pub(crate) fn set_tray_labels(app: AppHandle, labels: TrayLabels) {
    let valid = |text: &str| !text.trim().is_empty() && text.chars().count() <= 80;
    if !(valid(&labels.open) && valid(&labels.quit) && valid(&labels.empty)) {
        return;
    }
    *app.state::<TrayLabelState>().0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = labels;
    sync_tray(&app);
}

/// Holt das Hauptfenster hervor, wenn es im Menüleisten-Betrieb verborgen ist.
///
/// Die Oberfläche ruft das vor Rückfragen und nach Fehlern. Ohne sichtbares
/// Fenster gingen Dialog und Meldung sonst unter. Im normalen Betrieb bleibt
/// alles, wie es ist – außer, das Fenster wurde beim Start für einen Link
/// zurückgehalten.
#[tauri::command]
pub(crate) fn reveal_main_window(app: AppHandle) {
    if !menu_bar_enabled(&app) && !withheld() {
        return;
    }
    let visible = app
        .get_webview_window("main")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    if !visible {
        show_main_window(&app);
    }
}

/// Zeigt das Hauptfenster und, falls nötig, wieder das Dock-Symbol.
pub(crate) fn show_main_window(app: &AppHandle) {
    LAUNCH_WINDOW.store(LAUNCH_SHOWN, Ordering::Release);
    set_dock_visible(app, true);
    #[cfg(target_os = "macos")]
    crate::restore_main_window(app);
    #[cfg(not(target_os = "macos"))]
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Verbirgt das Hauptfenster in die Menüleiste.
pub(crate) fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    refresh_dock(app);
}

/// Schaltet das Dock-Symbol über die Aktivierungsrichtlinie.
///
/// Verglichen wird mit der **tatsächlichen** Richtlinie, nicht mit einem
/// gemerkten Wert: Öffnet eine Schreibtisch-Verknüpfung `remotedesk://…`,
/// macht LaunchServices aus der Menüleisten-App eigenmächtig wieder eine
/// Dock-App („changing application type to Foreground from UIElement“,
/// gemessen im Systemprotokoll). Ein gemerkter Wert hielt das Symbol danach
/// für verborgen und ließ es stehen. Gleiche Werte werden nicht erneut
/// gesetzt, weil macOS jeden Wechsel mit einem Neuaufbau der Menüleiste quittiert.
fn set_dock_visible(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
            let Some(mtm) = objc2::MainThreadMarker::new() else { return };
            let (wanted, policy) = if visible {
                (NSApplicationActivationPolicy::Regular, tauri::ActivationPolicy::Regular)
            } else {
                (NSApplicationActivationPolicy::Accessory, tauri::ActivationPolicy::Accessory)
            };
            if NSApplication::sharedApplication(mtm).activationPolicy() != wanted {
                let _ = handle.set_activation_policy(policy);
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

/// Das Dock-Symbol fehlt nur im Menüleisten-Betrieb ohne offenes Fenster.
/// VNC- und Terminalfenster zählen mit, damit sie per Cmd-Tab erreichbar bleiben.
pub(crate) fn refresh_dock(app: &AppHandle) {
    let any_visible = app
        .webview_windows()
        .values()
        .any(|window| window.is_visible().unwrap_or(false));
    set_dock_visible(app, !menu_bar_enabled(app) || any_visible);
}

/// Behandelt Fensterereignisse, die der Menüleisten-Betrieb ändert.
pub(crate) fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    let app = window.app_handle();
    if !menu_bar_enabled(app) {
        return;
    }
    match event {
        tauri::WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
            // Die App läuft im Symbol weiter; laufende Sitzungen bleiben offen.
            api.prevent_close();
            hide_main_window(app);
        }
        tauri::WindowEvent::Focused(true) => set_dock_visible(app, true),
        tauri::WindowEvent::Destroyed => refresh_dock(app),
        _ => {}
    }
}

/// Reagiert auf Einträge des Symbolmenüs. Andere Menüeinträge, etwa die
/// Update-Prüfung, laufen an diesem Handler vorbei.
pub(crate) fn on_menu_event(app: &AppHandle, id: &str) {
    if let Some(profile) = id.strip_prefix(TRAY_CONNECT_PREFIX) {
        // Die Oberfläche verbindet, wie bei einem Klick auf „Verbinden“;
        // nur sie kennt den Weg für VNC- und Terminalfenster.
        let _ = app.emit_to("main", "tray-connect", profile.to_string());
    } else if id == TRAY_OPEN_ID {
        show_main_window(app);
    } else if id == TRAY_QUIT_ID {
        app.exit(0);
    }
}

fn protocol_label(protocol: &Protocol) -> &'static str {
    match protocol {
        Protocol::Rdp => "RDP",
        Protocol::Vnc => "VNC",
        Protocol::Ssh => "SSH",
        Protocol::Sftp => "SFTP",
        Protocol::Mosh => "Mosh",
        Protocol::S3 => "S3",
        Protocol::Swift => "Swift",
    }
}

/// Menütext für ein Profil. Das Menü deutet ein einzelnes `&` als
/// Tastaturkürzel und verschluckt es, deshalb wird es verdoppelt.
fn menu_entry_text(name: &str, protocol: &Protocol) -> String {
    let name = name.trim();
    let mut short: String = name.chars().take(MAX_MENU_NAME).collect();
    if name.chars().count() > MAX_MENU_NAME {
        short.push('…');
    }
    format!("{}  ·  {}", short.replace('&', "&&"), protocol_label(protocol))
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    let labels = app.state::<TrayLabelState>().0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
    let mut profiles = read_profiles().unwrap_or_default();
    profiles.retain(|profile| !matches!(profile.protocol, Protocol::S3 | Protocol::Swift));
    profiles.sort_by_cached_key(|profile| profile.name.to_lowercase());

    let menu = Menu::new(app)?;
    if profiles.is_empty() {
        menu.append(&MenuItem::with_id(app, TRAY_EMPTY_ID, &labels.empty, false, None::<&str>)?)?;
    }
    for profile in &profiles {
        let id = format!("{TRAY_CONNECT_PREFIX}{}", profile.id);
        menu.append(&MenuItem::with_id(app, id, menu_entry_text(&profile.name, &profile.protocol), true, None::<&str>)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, TRAY_OPEN_ID, &labels.open, true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, TRAY_QUIT_ID, &labels.quit, true, None::<&str>)?)?;
    Ok(menu)
}

/// Bringt das Menüleisten-Symbol auf den aktuellen Stand: anlegen, Menü neu
/// aufbauen oder entfernen. Läuft stets auf dem Hauptthread, damit sich zwei
/// gleichzeitige Aufrufe nicht gegenseitig ein zweites Symbol anlegen.
pub(crate) fn sync_tray(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || sync_tray_now(&handle));
}

fn sync_tray_now(app: &AppHandle) {
    if !menu_bar_enabled(app) {
        // Das zurückgegebene Symbol wird hier verworfen; damit verschwindet es.
        drop(app.remove_tray_by_id(TRAY_ID));
        return;
    }
    let Ok(menu) = build_tray_menu(app) else { return };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_menu(Some(menu));
        return;
    }
    let mut builder = tauri::tray::TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("RemoteDeskRDP")
        .icon_as_template(true)
        .show_menu_on_left_click(true);
    if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png")) {
        builder = builder.icon(icon);
    }
    let _ = builder.build(app);
}

/// Start bei der Anmeldung über einen eigenen LaunchAgent.
///
/// Der Agent startet die App über `open`, also durch LaunchServices. Ein
/// direkter Start der Binary durch launchd machte launchd zum verantwortlichen
/// Prozess, und macOS ordnete die Freigabe „Lokales Netzwerk“ falsch zu.
/// Geladen wird der Agent erst bei der nächsten Anmeldung; ein sofortiges
/// `launchctl bootstrap` würde die App ein zweites Mal starten.
pub(crate) mod login_item {
    #[cfg(target_os = "macos")]
    use std::path::{Path, PathBuf};

    #[cfg(target_os = "macos")]
    const LABEL: &str = "com.nojan.remotedesk.login";

    #[cfg(target_os = "macos")]
    fn agent_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join("Library/LaunchAgents").join(format!("{LABEL}.plist")))
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn agent_plist(bundle: &Path, identifier: &str) -> String {
        let bundle = crate::xml_escape(&bundle.to_string_lossy());
        let identifier = crate::xml_escape(identifier);
        let background = super::BACKGROUND_ARGUMENT;
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{LABEL}</string>
	<key>ProgramArguments</key>
	<array>
		<string>/usr/bin/open</string>
		<string>-a</string>
		<string>{bundle}</string>
		<string>--args</string>
		<string>{background}</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>LimitLoadToSessionType</key>
	<string>Aqua</string>
	<key>AssociatedBundleIdentifiers</key>
	<array>
		<string>{identifier}</string>
	</array>
</dict>
</plist>
"#
        )
    }

    #[cfg(target_os = "macos")]
    fn current_bundle() -> Option<PathBuf> {
        std::env::current_exe().ok().as_deref().and_then(crate::bundle_path)
    }

    pub(crate) fn enabled() -> bool {
        #[cfg(target_os = "macos")]
        return agent_path().is_some_and(|path| path.exists());
        #[cfg(not(target_os = "macos"))]
        false
    }

    #[cfg(target_os = "macos")]
    fn write(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
        // Ohne Bundle, etwa beim Entwicklungsstart, gäbe es nichts zu öffnen.
        let bundle = current_bundle().ok_or_else(|| "err.loginItemNoBundle".to_string())?;
        let content = agent_plist(&bundle, &app.config().identifier);
        if std::fs::read_to_string(path).is_ok_and(|existing| existing == content) {
            return Ok(());
        }
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory).map_err(|e| crate::err1("err.loginItemWrite", e))?;
        }
        std::fs::write(path, content).map_err(|e| crate::err1("err.loginItemWrite", e))
    }

    pub(crate) fn set(app: &tauri::AppHandle, enable: bool) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let path = agent_path().ok_or_else(|| "err.configDirMissing".to_string())?;
            if enable {
                return write(app, &path);
            }
            // Nach einer Anmeldung ist der Agent noch geladen. Er läuft nicht
            // dauerhaft, soll aber ohne Abmeldung verschwinden.
            // SAFETY: getuid(2) ist immer erfolgreich und hat keine Nebenwirkungen.
            let uid = unsafe { libc::getuid() };
            let _ = std::process::Command::new("/bin/launchctl")
                .arg("bootout")
                .arg(format!("gui/{uid}/{LABEL}"))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            match std::fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(crate::err1("err.loginItemWrite", error)),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app, enable);
            Err("err.loginItemUnsupported".into())
        }
    }

    /// Hält den Agenten auf das aktuelle Bundle gerichtet, falls die App
    /// verschoben wurde. Ein abgeschalteter Anmeldestart bleibt aus.
    pub(crate) fn refresh(app: &tauri::AppHandle) {
        #[cfg(target_os = "macos")]
        if let Some(path) = agent_path().filter(|path| path.exists()) {
            if current_bundle().is_some() {
                let _ = write(app, &path);
            }
        }
        #[cfg(not(target_os = "macos"))]
        let _ = app;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_to_off_and_tolerate_unknown_fields() {
        assert_eq!(StoredSettings::default(), StoredSettings { menu_bar: false, desklet_at_startup: false });
        let parsed: StoredSettings = serde_json::from_str("{}").unwrap();
        assert!(!parsed.menu_bar, "fehlender Schlüssel bleibt aus");
        let parsed: StoredSettings = serde_json::from_str(r#"{"menuBar":true,"future":1}"#).unwrap();
        assert!(parsed.menu_bar);
        assert!(!parsed.desklet_at_startup, "bestehende Einstellungen aktivieren das Desklet nicht");
    }

    #[test]
    fn desklet_startup_option_is_preserved_with_menu_bar_settings() {
        for menu_bar in [false, true] {
            for desklet_at_startup in [false, true] {
                let settings = StoredSettings { menu_bar, desklet_at_startup };
                let json = serde_json::to_string(&settings).unwrap();
                assert!(json.contains("deskletAtStartup"));
                assert_eq!(serde_json::from_str::<StoredSettings>(&json).unwrap(), settings);
            }
        }
    }

    #[test]
    fn menu_entries_keep_ampersands_and_stay_short() {
        assert_eq!(menu_entry_text("Büro & Labor", &Protocol::Rdp), "Büro && Labor  ·  RDP");
        let long = "x".repeat(MAX_MENU_NAME + 10);
        let text = menu_entry_text(&long, &Protocol::Ssh);
        assert!(text.starts_with(&format!("{}…", "x".repeat(MAX_MENU_NAME))), "{text}");
        assert_eq!(menu_entry_text("Kurz", &Protocol::Vnc), "Kurz  ·  VNC");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn login_agent_opens_the_bundle_through_launch_services() {
        let plist = login_item::agent_plist(std::path::Path::new("/Applications/Remote & Desk.app"), "com.nojan.remotedesk");
        assert!(plist.contains("<string>/usr/bin/open</string>"));
        assert!(plist.contains("<string>/Applications/Remote &amp; Desk.app</string>"), "Pfad wird maskiert");
        assert!(plist.contains("<string>--background</string>"));
        assert!(plist.contains("<string>com.nojan.remotedesk</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>\n\t<true/>"));
        // plutil prüft, ob launchd die Datei überhaupt lesen kann.
        let file = std::env::temp_dir().join(format!("remotedesk-login-{}.plist", std::process::id()));
        std::fs::write(&file, &plist).unwrap();
        let status = std::process::Command::new("/usr/bin/plutil").arg("-lint").arg(&file).output().unwrap();
        let _ = std::fs::remove_file(&file);
        assert!(status.status.success(), "{}", String::from_utf8_lossy(&status.stdout));
    }
}
