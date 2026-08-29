use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use quick_xml::de::from_str as xml_from_str;
use reqwest::blocking::{Client as HttpClient, RequestBuilder};
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{Emitter, Manager};

// Die App heisst seit der Umbenennung RemoteDeskRDP. Die folgenden drei
// Bezeichner behalten trotzdem den alten Namen: An ihnen haengen der
// Profilordner, die Kennwoerter im Schluesselbund und die Umgebungsvariable
// aus der Dokumentation. Eine Umbenennung wuerde bestehende Profile und
// Kennwoerter unauffindbar machen.
const PROFILE_DIRECTORY: &str = "RemoteDesk";
const PROFILE_FILE: &str = "profiles.json";
const KEYCHAIN_SERVICE: &str = "com.nojan.remotedesk.password";
// Eigener Dienst, damit das Gatewaykennwort neben dem Sitzungskennwort unter
// derselben Profil-ID liegen kann. KEYCHAIN_SERVICE darf nicht angetastet
// werden -- daran hängen die bereits gespeicherten Kennwörter.
const GATEWAY_KEYCHAIN_SERVICE: &str = "com.nojan.remotedesk.gateway";
/// S3 Secret Access Key und Swift-Kennwort sind nicht mit RDP/VNC-Passwörtern
/// vermischt. So kann ein Objekt-Storage-Profil niemals ein Sitzungskennwort
/// eines älteren Profils übernehmen.
const OBJECT_STORAGE_KEYCHAIN_SERVICE: &str = "com.nojan.remotedesk.object-storage";
const DEFAULT_GATEWAY_PORT: u16 = 443;
const DEFAULT_VNC_PORT: u16 = 5900;
const DEFAULT_SSH_PORT: u16 = 22;
const DEFAULT_SSH_TERMINAL: &str = "xterm-256color";
const DEFAULT_SWIFT_IDENTITY_PATH: &str = "/identity/v3";
const BACKEND_OVERRIDE: &str = "REMOTEDESK_RDP_EXECUTABLE";
static PROFILE_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static SSH_SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

// Fehler gehen als Code an die Oberfläche, nicht als fertiger Satz -- sonst
// bliebe die Meldung deutsch, auch wenn der Nutzer auf Englisch umschaltet.
// Das Frontend schlägt den Code in seinen Wörterbüchern nach.
// Parameter hängen hinter dem Unit-Separator und ersetzen dort {0}, {1} …

/// Fehlercode mit einem Parameter, etwa einer Meldung des Betriebssystems.
fn err1(code: &str, argument: impl std::fmt::Display) -> String {
    format!("{code}\u{1f}{argument}")
}
const DEFAULT_WIDTH: u16 = 1600;
const DEFAULT_HEIGHT: u16 = 1000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoteProfile {
    id: String,
    name: String,
    /// Fehlte dieses Feld in älteren Dateien, war das Profil immer ein
    /// FreeRDP-Profil. Der Default hält diese Profile unverändert lauffähig.
    #[serde(default)]
    protocol: Protocol,
    host: String,
    username: String,
    domain: String,
    rdp_tcp_port: u16,
    rdp_udp_port: u16,
    #[serde(default = "default_vnc_port")]
    vnc_port: u16,
    #[serde(default = "default_ssh_port")]
    ssh_port: u16,
    #[serde(default = "default_ssh_terminal")]
    ssh_terminal: String,
    /// X11-Fenster werden nur für reguläre SSH-Sitzungen weitergeleitet.
    #[serde(default)]
    x11_forwarding: bool,
    /// Optionaler, bewusst vom Profilinhaber bestimmter Befehl auf dem Server.
    /// Er wird erst nach dem SSH-Ziel übergeben und daher nie lokal ausgeführt.
    #[serde(default)]
    x11_command: String,
    /// Nur auf ausdrücklichen Wunsch beendet die App XQuartz nach der
    /// zugehörigen X11-Sitzung. Der Standard schützt andere X11-Programme.
    #[serde(default)]
    x11_close_xquartz: bool,
    /// Gemeinsame Objekt-Storage-Angaben. Die Geheimnisse selbst liegen nie in
    /// profiles.json, sondern ausschließlich im macOS-Schlüsselbund.
    #[serde(default)]
    object_endpoint: String,
    #[serde(default)]
    object_region: String,
    #[serde(default)]
    object_access_key: String,
    #[serde(default)]
    object_container: String,
    #[serde(default = "default_true")]
    object_path_style: bool,
    #[serde(default)]
    swift_project: String,
    #[serde(default)]
    swift_user_domain: String,
    #[serde(default)]
    swift_project_domain: String,
    /// Keystone-Endpunkte sind anbieterspezifisch. Er bleibt getrennt vom
    /// Standort, damit zum Beispiel /v3 oder /identity/v3 auswählbar ist.
    #[serde(default = "default_swift_identity_path")]
    swift_identity_path: String,
    #[serde(default)]
    swift_auth_version: SwiftAuthVersion,
    udp_preferred: bool,
    clipboard: bool,
    audio: bool,
    // Ältere Profile kennen die Anzeigefelder noch nicht.
    #[serde(default)]
    display_mode: DisplayMode,
    #[serde(default = "default_width")]
    width: u16,
    #[serde(default = "default_height")]
    height: u16,
    #[serde(default)]
    resize_behavior: ResizeBehavior,
    #[serde(default)]
    color_depth: ColorDepth,
    #[serde(default)]
    shared_folders: Vec<SharedFolder>,
    // Ältere Profile kennen die Druckerfreigabe noch nicht.
    #[serde(default)]
    printer: bool,
    #[serde(default)]
    smartcard: bool,
    #[serde(default)]
    video: bool,
    /// Wiederverbindung nach einem kurzen Aussetzer.
    ///
    /// In FreeRDP ist `AutoReconnectionEnabled` per Vorgabe **FALSE**
    /// (libfreerdp/core/settings.c, Z. 1215); ein einziger verlorener Takt
    /// beendet die Sitzung sonst endgültig. Bestandsprofile bekommen die
    /// Option daher **eingeschaltet** – sie wurde genau deswegen eingeführt.
    #[serde(default = "default_true")]
    auto_reconnect: bool,
    // Gateway (RD Gateway / RDS-Gatewayserver). Ältere Profile kennen es nicht.
    #[serde(default)]
    gateway_enabled: bool,
    #[serde(default)]
    gateway_host: String,
    #[serde(default = "default_gateway_port")]
    gateway_port: u16,
    #[serde(default)]
    gateway_username: String,
    #[serde(default)]
    gateway_domain: String,
    certificate_mode: CertificateMode,
    updated_at: String,
}

/// Das Protokoll bestimmt sowohl die Eingabemaske als auch den gestarteten
/// Client. VNC wird im Hauptfenster mit dem eingebetteten Standard-RFB-Viewer
/// dargestellt.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Protocol {
    #[default]
    Rdp,
    Vnc,
    Ssh,
    Sftp,
    Mosh,
    S3,
    Swift,
}

/// v2 bleibt ausschließlich für ältere Clouds auswählbar. Aktuelle Keystone-
/// Installationen unterstützen laut OpenStack nur noch v3; neue Profile
/// starten deshalb zwingend mit v3.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum SwiftAuthVersion {
    V2,
    #[default]
    V3,
}

fn default_width() -> u16 {
    DEFAULT_WIDTH
}

fn default_gateway_port() -> u16 {
    DEFAULT_GATEWAY_PORT
}

fn default_vnc_port() -> u16 {
    DEFAULT_VNC_PORT
}

fn default_ssh_port() -> u16 { DEFAULT_SSH_PORT }
fn default_ssh_terminal() -> String { DEFAULT_SSH_TERMINAL.into() }
fn default_swift_identity_path() -> String { DEFAULT_SWIFT_IDENTITY_PATH.into() }

fn default_true() -> bool {
    true
}

fn default_height() -> u16 {
    DEFAULT_HEIGHT
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum DisplayMode {
    /// Fenster mit der im Profil hinterlegten Größe.
    #[default]
    Window,
    /// Fenster füllt die nutzbare Bildschirmfläche ohne Menü- und Dock-Bereich.
    WorkArea,
    Fullscreen,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ResizeBehavior {
    /// Der Server passt die Auflösung an die Fenstergröße an (benötigt Serverunterstützung).
    #[default]
    Dynamic,
    /// Das Bild wird auf die Fenstergröße skaliert; funktioniert mit jedem Server.
    Scale,
    /// Feste Auflösung, das Fenster bleibt unveränderlich.
    Fixed,
}

/// Ein als Laufwerk in die Sitzung gereichter Ordner.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SharedFolder {
    name: String,
    path: String,
}

/// FreeRDP nimmt nur 32, 24, 16, 15 und 8 entgegen und bricht sonst mit einem
/// Argumentfehler ab.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
enum ColorDepth {
    /// Ohne `/bpp` handeln Client und Server die Farbtiefe selbst aus.
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "32")]
    Bits32,
    #[serde(rename = "24")]
    Bits24,
    #[serde(rename = "16")]
    Bits16,
    #[serde(rename = "15")]
    Bits15,
    #[serde(rename = "8")]
    Bits8,
}

impl ColorDepth {
    fn bits(self) -> Option<u8> {
        match self {
            ColorDepth::Auto => None,
            ColorDepth::Bits32 => Some(32),
            ColorDepth::Bits24 => Some(24),
            ColorDepth::Bits16 => Some(16),
            ColorDepth::Bits15 => Some(15),
            ColorDepth::Bits8 => Some(8),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum CertificateMode {
    Prompt,
    Tofu,
    Ignore,
}

fn config_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "Application-Support-Ordner nicht gefunden".to_string())?;
    Ok(base.join(PROFILE_DIRECTORY))
}

fn profile_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join(PROFILE_FILE))
}

fn read_profiles() -> Result<Vec<RemoteProfile>, String> {
    let path = profile_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| err1("err.profileFileUnreadable", format!("{}: {e}", path.display())))?;
    serde_json::from_str(&data).map_err(|e| err1("err.profileFileInvalid", e))
}

fn write_profiles(profiles: &[RemoteProfile]) -> Result<(), String> {
    let directory = config_dir()?;
    fs::create_dir_all(&directory)
        .map_err(|e| err1("err.profileDirectory", format!("{}: {e}", directory.display())))?;
    let serialized = serde_json::to_string_pretty(profiles).map_err(|e| err1("err.profileWrite", e))?;
    // Jede Schreiboperation erhält eine eigene Datei. Ein fester Name würde
    // parallele Instanzen beim Schreiben oder Umbenennen kollidieren lassen.
    let (temporary, mut file) = (0..100)
        .find_map(|_| {
            let sequence = PROFILE_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(".profiles.json.{}.{}.tmp", std::process::id(), sequence));
            match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => Some(Ok((path, file))),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error)),
            }
        })
        .unwrap_or_else(|| Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "keine freie temporäre Profildatei")))
        .map_err(|e| err1("err.profileWrite", e))?;
    if let Err(error) = file.write_all(serialized.as_bytes()).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(err1("err.profileWrite", error));
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, profile_path()?) {
        let _ = fs::remove_file(&temporary);
        return Err(err1("err.profileWrite", error));
    }
    Ok(())
}

/// Die Optionen reisen zeilenweise über `/args-from:stdin`; Umbrüche würden
/// daraus zusätzliche FreeRDP-Argumente machen.
fn ensure_single_line(value: &str) -> Result<(), String> {
    if value.contains('\n') || value.contains('\r') {
        Err("err.argumentLineBreak".into())
    } else {
        Ok(())
    }
}

fn validate_shared_folder(folder: &SharedFolder) -> Result<(), String> {
    let name = folder.name.trim();
    let path = folder.path.trim();
    if name.is_empty() || path.is_empty() {
        return Err("err.shareNeedsNameAndPath".into());
    }
    ensure_single_line(name)?;
    ensure_single_line(path)?;
    if name.contains(',') || path.contains(',') {
        return Err(err1("err.shareNoComma", name));
    }
    Ok(())
}

fn validate_profile(profile: &RemoteProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() || profile.name.trim().is_empty() {
        return Err("err.profileNeedsIdAndName".into());
    }
    let is_object_storage = matches!(profile.protocol, Protocol::S3 | Protocol::Swift);
    if (!is_object_storage && profile.host.trim().is_empty()) || profile.host.chars().any(char::is_whitespace) {
        return Err("err.hostRequired".into());
    }
    ensure_single_line(&profile.username)?;
    ensure_single_line(&profile.domain)?;
    for folder in &profile.shared_folders {
        validate_shared_folder(folder)?;
    }
    // u16 stellt sicher, dass ein gesetzter Port im zulässigen Bereich liegt.
    if profile.protocol == Protocol::Rdp && (profile.rdp_tcp_port == 0 || profile.rdp_udp_port == 0) {
        return Err("err.portRange".into());
    }
    if profile.protocol == Protocol::Vnc && profile.vnc_port == 0 {
        return Err("err.portRange".into());
    }
    if matches!(profile.protocol, Protocol::Ssh | Protocol::Sftp | Protocol::Mosh) && profile.ssh_port == 0 {
        return Err("err.portRange".into());
    }
    if matches!(profile.protocol, Protocol::Ssh | Protocol::Sftp | Protocol::Mosh) {
        let terminal = profile.ssh_terminal.trim();
        if terminal.is_empty() || terminal.len() > 64 || !terminal.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-')) {
            return Err("err.sshTerminalInvalid".into());
        }
    }
    ensure_single_line(&profile.x11_command)?;
    if profile.x11_command.len() > 1024 {
        return Err("err.x11CommandInvalid".into());
    }
    if is_object_storage {
        let endpoint = profile.object_endpoint.trim();
        let valid_endpoint = reqwest::Url::parse(endpoint)
            .ok()
            .is_some_and(|url| matches!(url.scheme(), "https" | "http") && url.host_str().is_some());
        if !valid_endpoint { return Err("err.objectEndpointInvalid".into()); }
        for value in [
            &profile.object_region, &profile.object_access_key, &profile.object_container,
            &profile.swift_project, &profile.swift_user_domain, &profile.swift_project_domain, &profile.swift_identity_path,
        ] { ensure_single_line(value)?; }
        if profile.protocol == Protocol::S3 {
            if profile.object_region.trim().is_empty() || profile.object_access_key.trim().is_empty() {
                return Err("err.s3CredentialsRequired".into());
            }
        } else if profile.username.trim().is_empty() || profile.swift_project.trim().is_empty() {
            return Err("err.swiftCredentialsRequired".into());
        } else if !profile.swift_identity_path.starts_with('/') || profile.swift_identity_path.len() > 512 || profile.swift_identity_path.split('/').any(|segment| segment == "..") {
            return Err("err.swiftPathInvalid".into());
        }
    }
    if profile.protocol == Protocol::Rdp && profile.display_mode == DisplayMode::Window {
        // FreeRDP verlangt Vielfache von 2 und lehnt Winzformate ab.
        if profile.width < 640 || profile.height < 480 {
            return Err("err.windowTooSmall".into());
        }
        if profile.width % 2 != 0 || profile.height % 2 != 0 {
            return Err("err.oddDimensions".into());
        }
    }
    if profile.protocol == Protocol::Rdp && profile.gateway_enabled {
        let host = profile.gateway_host.trim();
        if host.is_empty() || host.chars().any(char::is_whitespace) {
            return Err("err.gatewayHostRequired".into());
        }
        if host.contains(':') {
            return Err("err.gatewayHostColon".into());
        }
        if profile.gateway_port == 0 {
            return Err("err.gatewayPortRange".into());
        }
        ensure_single_line(&profile.gateway_username)?;
        ensure_single_line(&profile.gateway_domain)?;
    }
    Ok(())
}

fn keychain_account(profile_id: &str) -> Result<&str, String> {
    if profile_id.trim().is_empty() || profile_id.chars().any(char::is_whitespace) {
        return Err("err.badProfileId".into());
    }
    Ok(profile_id)
}

#[tauri::command]
fn save_password(profile_id: String, password: String) -> Result<(), String> {
    let account = keychain_account(&profile_id)?;
    if password.is_empty() || password.contains('\n') || password.contains('\r') {
        return Err("err.emptyPassword".into());
    }
    #[cfg(target_os = "macos")]
    {
        security_framework::passwords::set_generic_password(KEYCHAIN_SERVICE, account, password.as_bytes())
            .map_err(|e| err1("err.keychain", e))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (account, password);
        Err("err.keychainOnlyMacos".into())
    }
}

fn profile_password(profile_id: &str) -> Result<Option<String>, String> {
    let account = keychain_account(profile_id)?;
    #[cfg(target_os = "macos")]
    {
        match security_framework::passwords::get_generic_password(KEYCHAIN_SERVICE, account) {
            Ok(secret) => String::from_utf8(secret).map(Some).map_err(|_| "Ungültiges Kennwort im Schlüsselbund".to_string()),
            Err(_) => Ok(None),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        Ok(None)
    }
}

#[tauri::command]
fn load_password(profile_id: String) -> Result<Option<String>, String> {
    profile_password(&profile_id)
}

/// Entfernt nur das Sitzungskennwort. Ein fehlender Eintrag ist kein Fehler:
/// das Passwortfeld darf ohne Vorbedingung geleert werden.
#[tauri::command]
fn forget_password(profile_id: String) -> Result<(), String> {
    let account = keychain_account(&profile_id)?;
    #[cfg(target_os = "macos")]
    {
        match security_framework::passwords::delete_generic_password(KEYCHAIN_SERVICE, account) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == security_framework_sys::base::errSecItemNotFound => Ok(()),
            Err(error) => Err(err1("err.keychain", error)),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        Err("err.keychainOnlyMacos".into())
    }
}

#[tauri::command]
fn save_gateway_password(profile_id: String, password: String) -> Result<(), String> {
    let account = keychain_account(&profile_id)?;
    if password.is_empty() || password.contains('\n') || password.contains('\r') {
        return Err("err.emptyPassword".into());
    }
    #[cfg(target_os = "macos")]
    {
        security_framework::passwords::set_generic_password(GATEWAY_KEYCHAIN_SERVICE, account, password.as_bytes())
            .map_err(|e| err1("err.keychain", e))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (account, password);
        Err("err.keychainOnlyMacos".into())
    }
}

fn gateway_password(profile_id: &str) -> Result<Option<String>, String> {
    let account = keychain_account(profile_id)?;
    #[cfg(target_os = "macos")]
    {
        match security_framework::passwords::get_generic_password(GATEWAY_KEYCHAIN_SERVICE, account) {
            Ok(secret) => String::from_utf8(secret).map(Some).map_err(|_| "Ungültiges Kennwort im Schlüsselbund".to_string()),
            Err(_) => Ok(None),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        Ok(None)
    }
}

#[tauri::command]
fn load_gateway_password(profile_id: String) -> Result<Option<String>, String> {
    gateway_password(&profile_id)
}

/// Entfernt nur das Gatewaykennwort. Ein fehlender Eintrag ist kein Fehler.
#[tauri::command]
fn forget_gateway_password(profile_id: String) -> Result<(), String> {
    let account = keychain_account(&profile_id)?;
    #[cfg(target_os = "macos")]
    {
        match security_framework::passwords::delete_generic_password(GATEWAY_KEYCHAIN_SERVICE, account) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == security_framework_sys::base::errSecItemNotFound => Ok(()),
            Err(error) => Err(err1("err.keychain", error)),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        Err("err.keychainOnlyMacos".into())
    }
}

/// Entfernt beide Kennwörter eines Profils aus dem Schlüsselbund. Ein fehlender
/// Eintrag ist der Normalfall -- etwa wenn nie ein Gatewaykennwort gesetzt
/// wurde -- und deshalb kein Fehler.
fn forget_passwords(profile_id: &str) {
    let Ok(account) = keychain_account(profile_id) else {
        return;
    };
    #[cfg(target_os = "macos")]
    for service in [KEYCHAIN_SERVICE, GATEWAY_KEYCHAIN_SERVICE, OBJECT_STORAGE_KEYCHAIN_SERVICE] {
        let _ = security_framework::passwords::delete_generic_password(service, account);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = account;
}

#[tauri::command]
fn save_object_secret(profile_id: String, secret: String) -> Result<(), String> {
    let account = keychain_account(&profile_id)?;
    if secret.is_empty() || secret.contains('\n') || secret.contains('\r') {
        return Err("err.emptyPassword".into());
    }
    #[cfg(target_os = "macos")]
    {
        security_framework::passwords::set_generic_password(OBJECT_STORAGE_KEYCHAIN_SERVICE, account, secret.as_bytes())
            .map_err(|error| err1("err.keychain", error))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (account, secret);
        Err("err.keychainOnlyMacos".into())
    }
}

fn object_secret(profile_id: &str) -> Result<Option<String>, String> {
    let account = keychain_account(profile_id)?;
    #[cfg(target_os = "macos")]
    {
        match security_framework::passwords::get_generic_password(OBJECT_STORAGE_KEYCHAIN_SERVICE, account) {
            Ok(secret) => String::from_utf8(secret).map(Some).map_err(|_| "Ungültiges Objekt-Storage-Geheimnis im Schlüsselbund".to_string()),
            Err(_) => Ok(None),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        Ok(None)
    }
}

#[tauri::command]
fn load_object_secret(profile_id: String) -> Result<Option<String>, String> {
    object_secret(&profile_id)
}

#[tauri::command]
fn forget_object_secret(profile_id: String) -> Result<(), String> {
    let account = keychain_account(&profile_id)?;
    #[cfg(target_os = "macos")]
    {
        match security_framework::passwords::delete_generic_password(OBJECT_STORAGE_KEYCHAIN_SERVICE, account) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == security_framework_sys::base::errSecItemNotFound => Ok(()),
            Err(error) => Err(err1("err.keychain", error)),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        Err("err.keychainOnlyMacos".into())
    }
}

/// Serialisiert Read-Modify-Write-Zugriffe auf die gemeinsame Profildatei.
#[derive(Default)]
struct ProfileStore(Mutex<()>);

fn lock_profiles(store: &ProfileStore) -> Result<std::sync::MutexGuard<'_, ()>, String> {
    store.0.lock().map_err(|_| "err.profileLock".into())
}

#[tauri::command]
fn list_profiles(store: tauri::State<'_, ProfileStore>) -> Result<Vec<RemoteProfile>, String> {
    let _guard = lock_profiles(&store)?;
    let mut profiles = read_profiles()?;
    // Objekt-Speicher wird seit der Übergabe an DualBeam nicht mehr von
    // RemoteDeskRDP angeboten. Alte Einträge bleiben als reversible Daten im
    // Profilarchiv, erscheinen aber nicht mehr in dieser App.
    profiles.retain(|profile| !matches!(profile.protocol, Protocol::S3 | Protocol::Swift));
    profiles.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(profiles)
}

#[tauri::command]
fn save_profile(profile: RemoteProfile, store: tauri::State<'_, ProfileStore>) -> Result<RemoteProfile, String> {
    let _guard = lock_profiles(&store)?;
    if matches!(profile.protocol, Protocol::S3 | Protocol::Swift) {
        return Err("err.objectStorageMoved".into());
    }
    validate_profile(&profile)?;
    let mut profiles = read_profiles()?;
    if let Some(index) = profiles.iter().position(|item| item.id == profile.id) {
        profiles[index] = profile.clone();
    } else {
        profiles.push(profile.clone());
    }
    write_profiles(&profiles)?;
    Ok(profile)
}

#[tauri::command]
fn delete_profile(id: String, store: tauri::State<'_, ProfileStore>) -> Result<(), String> {
    let _guard = lock_profiles(&store)?;
    let mut profiles = read_profiles()?;
    profiles.retain(|profile| profile.id != id);
    write_profiles(&profiles)?;
    // Erst nach dem erfolgreichen Schreiben: schlägt das Schreiben fehl, bleibt
    // das Profil bestehen und soll sein Kennwort behalten.
    forget_passwords(&id);
    Ok(())
}

fn executable_at(path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    path.is_file().then(|| path.to_path_buf())
}

const BUNDLED_BACKEND_DIRECTORY: &str = "resources/freerdp/MacFreeRDP.app/Contents/MacOS";

/// Die mitgelieferten Clients. Der SDL-Client ist die erste Wahl: Nur er
/// beherrscht den Display-Control-Kanal und kann damit die Serverauflösung
/// beim Vergrößern des Fensters wirklich mitziehen. Sein Metal-Renderer
/// blockiert unter macOS in `[CAMetalLayer nextDrawable]`, deshalb startet
/// RemoteDeskRDP ihn ausschließlich mit dem OpenGL-Renderer. Der native
/// Cocoa-Client dient als Rückfall; er kann nur skalieren.
const BUNDLED_BACKENDS: [&str; 2] = ["sdl-freerdp", "MacFreeRDP"];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Backend {
    /// Nativer Cocoa-Client ohne Display-Control-Kanal.
    Cocoa,
    Sdl,
    X11,
}

fn backend_kind(binary: &Path) -> Backend {
    match binary.file_name().and_then(|name| name.to_str()) {
        Some("MacFreeRDP") => Backend::Cocoa,
        Some("xfreerdp") => Backend::X11,
        _ => Backend::Sdl,
    }
}

fn bundled_backends(app: &tauri::AppHandle) -> Vec<PathBuf> {
    use tauri::Manager;
    let mut roots = Vec::new();
    if let Ok(path) = app
        .path()
        .resolve(BUNDLED_BACKEND_DIRECTORY, tauri::path::BaseDirectory::Resource)
    {
        roots.push(path);
    }
    // Während `tauri dev` liegen die Ressourcen noch im Projektverzeichnis.
    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors() {
            roots.push(ancestor.join(BUNDLED_BACKEND_DIRECTORY));
            roots.push(ancestor.join("src-tauri").join(BUNDLED_BACKEND_DIRECTORY));
        }
    }
    BUNDLED_BACKENDS
        .iter()
        .flat_map(|name| roots.iter().map(move |root| root.join(name)))
        .collect()
}

fn homebrew_freerdp() -> Vec<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/opt/homebrew/bin/sdl-freerdp"),
        PathBuf::from("/usr/local/bin/sdl-freerdp"),
    ];
    // xfreerdp zeichnet ausschließlich über X11.
    if Path::new("/Applications/Utilities/XQuartz.app").is_dir() {
        candidates.push(PathBuf::from("/opt/homebrew/bin/xfreerdp"));
        candidates.push(PathBuf::from("/usr/local/bin/xfreerdp"));
    }
    candidates
}

fn rdp_executable(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if let Some(value) = std::env::var_os(BACKEND_OVERRIDE) {
        let path = PathBuf::from(value);
        return executable_at(&path)
            .ok_or_else(|| format!("{BACKEND_OVERRIDE} verweist nicht auf eine Datei: {}", path.display()));
    }
    bundled_backends(app)
        .into_iter()
        .chain(homebrew_freerdp())
        .find_map(executable_at)
        .ok_or_else(|| {
            "Kein RDP-Backend gefunden. Erstelle es mit `npm run build:rdp-backend` oder setze \
             REMOTEDESK_RDP_EXECUTABLE auf eine FreeRDP-Binärdatei."
                .into()
        })
}

fn rdp_target(host: &str, port: u16) -> String {
    // FreeRDP erwartet IPv6-Adressen in eckigen Klammern.
    let host = host.trim();
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

/// Anzeigeoptionen für FreeRDP. `/smart-sizing` und `+dynamic-resolution`
/// schließen sich gegenseitig aus, FreeRDP bricht sonst beim Parsen ab.
fn display_arguments(profile: &RemoteProfile, backend: Backend) -> Vec<String> {
    let mut arguments = Vec::new();
    match profile.display_mode {
        DisplayMode::Fullscreen => arguments.push("+f".into()),
        DisplayMode::WorkArea => arguments.push("+workarea".into()),
        DisplayMode::Window => arguments.push(format!("/size:{}x{}", profile.width, profile.height)),
    }
    if let Some(bits) = profile.color_depth.bits() {
        arguments.push(format!("/bpp:{bits}"));
    }
    match profile.resize_behavior {
        // Der Cocoa-Client kennt den Display-Control-Kanal nicht. Ohne diesen
        // Rückfall bliebe das Fenster dort in fester Auflösung stehen.
        ResizeBehavior::Dynamic if backend == Backend::Cocoa => arguments.push("/smart-sizing".into()),
        ResizeBehavior::Dynamic => arguments.push("+dynamic-resolution".into()),
        ResizeBehavior::Scale => arguments.push("/smart-sizing".into()),
        ResizeBehavior::Fixed => {}
    }
    arguments
}

/// Der Titel des Sitzungsfensters.
///
/// Ohne `/t:` setzt der SDL-Client "FreeRDP: <Host>" (sdl_context.cpp, Z. 499).
/// Der Profilname sagt dem Nutzer mehr, und im Fensterwechsler stehen mehrere
/// Sitzungen dann unterscheidbar nebeneinander.
///
/// Die Argumente reisen zeilenweise über `/args-from:stdin`. Ein Zeilenumbruch
/// im Namen würde die Liste zerreissen, deshalb werden Steuerzeichen ersetzt.
fn window_title(profile: &RemoteProfile) -> String {
    let cleaned: String = profile
        .name
        .trim()
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() { profile.host.trim().to_string() } else { cleaned }
}

fn rdp_arguments(profile: &RemoteProfile, backend: Backend) -> Result<Vec<String>, String> {
    validate_profile(profile)?;
    if profile.protocol != Protocol::Rdp {
        return Err("err.rdpProfileRequired".into());
    }
    let mut arguments = vec![format!("/v:{}", rdp_target(&profile.host, profile.rdp_tcp_port))];
    // Legt den Protokollumfang ausdrücklich fest, statt sich auf FreeRDPs
    // Vorgabe zu verlassen. Gemessen ist die Vorgabe bereits INFO -- dieselbe
    // Ausgabe mit und ohne Schalter. Der Schalter ändert also heute nichts;
    // er hält den Umfang nur fest, falls eine spätere FreeRDP-Fassung die
    // Vorgabe absenkt. Gebraucht werden genau zwei INFO-Zeilen:
    // "Network disconnect!" und "Attempting reconnect (n of 20)"
    // (client/common/client.c) -- ohne sie ist nach einem Abbruch nicht
    // erkennbar, ob überhaupt wiederverbunden wurde.
    arguments.push("/log-level:INFO".into());
    arguments.push(format!("/t:{}", window_title(profile)));
    if !profile.username.trim().is_empty() {
        arguments.push(format!("/u:{}", profile.username.trim()));
    }
    if !profile.domain.trim().is_empty() {
        arguments.push(format!("/d:{}", profile.domain.trim()));
    }
    match profile.certificate_mode {
        CertificateMode::Prompt => arguments.push("/cert:deny".into()),
        CertificateMode::Tofu => arguments.push("/cert:tofu".into()),
        CertificateMode::Ignore => arguments.push("/cert:ignore".into()),
    };
    arguments.extend(display_arguments(profile, backend));
    if profile.clipboard {
        arguments.push("+clipboard".into());
    }
    if profile.audio {
        arguments.push("/sound".into());
    }
    // Ohne Argument reicht FreeRDP alle über CUPS eingerichteten Drucker
    // weiter. Das Backend ist mit WITH_CUPS gebaut und gegen libcups
    // gelinkt; die Treiberzuordnung übernimmt die Gegenstelle.
    if profile.printer {
        arguments.push("/printer".into());
    }
    // winpr lädt PCSC zur Laufzeit aus /System/Library/Frameworks; ein
    // eigener Dienst wie pcscd unter Linux ist auf macOS nicht nötig.
    if profile.smartcard {
        arguments.push("/smartcard".into());
    }
    // Beschleunigt die Videowiedergabe *innerhalb* der Sitzung (MS-RDPEVOR).
    // Das ist keine Webcam-Weiterleitung -- die kann FreeRDP auf macOS nicht.
    if profile.video {
        arguments.push("/video".into());
    }
    arguments.extend(drive_arguments(profile)?);
    arguments.extend(gateway_arguments(profile)?);
    // Ohne diesen Schalter gibt FreeRDP bei einem kurzen Aussetzer sofort auf.
    // Wirksam wird er nur, wenn der Server beim Anmelden ein
    // Auto-Reconnect-Cookie geliefert hat (MS-RDPBCGR 2.2.4); andernfalls
    // bleibt er folgenlos. `AutoReconnectMaxRetries` steht bereits auf 20,
    // ein eigenes Feld dafür wäre nur zusätzliche Bedienlast.
    if profile.auto_reconnect {
        arguments.push("+auto-reconnect".into());
    }
    if profile.udp_preferred {
        arguments.extend(["+multitransport".into(), "/network:auto".into()]);
    } else {
        arguments.push("-multitransport".into());
    }
    if let Some(password) = profile_password(&profile.id)? {
        ensure_single_line(&password)?;
        arguments.push(format!("/p:{password}"));
    }
    Ok(arguments)
}

/// FreeRDP zerlegt den Wert von `/gateway:` an Kommas und entfernt danach
/// Maskierungen (`unescape`, client/common/cmdline.c). Gemessen am gebauten
/// Binary:
///   * rohes Komma      -> "Command line parsing failed"
///   * `\,` maskiert    -> angenommen
///   * Anführungszeichen brechen die Zerlegung ab; `parse_gateway_options`
///     verwirft das Gateway dann **stillschweigend** (`if (count == 0)
///     return TRUE;`) und die Verbindung geht direkt zum Ziel.
///   * `\"` bzw. `\'` maskiert -> angenommen
/// Ein still umgangenes Gateway wäre der schlimmste Ausgang, deshalb wird jedes
/// Sonderzeichen maskiert statt nur abgewiesen.
fn escape_gateway_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '\\' | ',' | '"' | '\'') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

/// Baut `/gateway:`. Bleiben Benutzer, Domäne und Kennwort leer, setzt FreeRDP
/// `GatewayUseSameCredentials = TRUE` (siehe `parse_gateway_host_option`) und
/// meldet das Gateway mit den Anmeldedaten der Sitzung an. Sobald eines der
/// Felder gefüllt ist, schaltet `parse_gateway_cred_option` auf FALSE.
fn gateway_arguments(profile: &RemoteProfile) -> Result<Vec<String>, String> {
    if !profile.gateway_enabled {
        return Ok(Vec::new());
    }
    let host = profile.gateway_host.trim();
    if host.is_empty() {
        return Err("err.gatewayHostRequired".into());
    }
    if host.chars().any(char::is_whitespace) {
        return Err("err.gatewayHostSpace".into());
    }
    // Der Port steht in einem eigenen Feld; ein Doppelpunkt im Namen ergäbe
    // sonst zwei Portangaben. IPv6-Adressen sind damit ausgeschlossen -- lieber
    // deutlich abweisen als still die falsche Gegenstelle ansprechen.
    if host.contains(':') {
        return Err("err.gatewayHostColon".into());
    }
    if profile.gateway_port == 0 {
        return Err("err.gatewayPortRange".into());
    }
    let mut options = vec![format!("g:{}:{}", escape_gateway_value(host), profile.gateway_port)];
    let username = profile.gateway_username.trim();
    if !username.is_empty() {
        ensure_single_line(username)?;
        options.push(format!("u:{}", escape_gateway_value(username)));
    }
    let domain = profile.gateway_domain.trim();
    if !domain.is_empty() {
        ensure_single_line(domain)?;
        options.push(format!("d:{}", escape_gateway_value(domain)));
    }
    if let Some(password) = gateway_password(&profile.id)? {
        ensure_single_line(&password)?;
        options.push(format!("p:{}", escape_gateway_value(&password)));
    }
    Ok(vec![format!("/gateway:{}", options.join(","))])
}

/// FreeRDP trennt Freigabename und Pfad mit einem Komma und kennt keine
/// Maskierung. Ein Komma im Namen verschöbe daher den Pfad, ein Komma im Pfad
/// schnitte ihn ab -- beides muss vorher auffallen statt still eine falsche
/// oder gar keine Freigabe zu erzeugen.
fn drive_arguments(profile: &RemoteProfile) -> Result<Vec<String>, String> {
    let mut arguments = Vec::new();
    for folder in &profile.shared_folders {
        validate_shared_folder(folder)?;
        let name = folder.name.trim();
        let path = folder.path.trim();
        arguments.push(format!("/drive:{name},{path}"));
    }
    Ok(arguments)
}

/// Liest FreeRDPs Fehlerausgabe fortlaufend mit – in den Speicher **und**, wenn
/// ein Pfad angegeben ist, in eine Datei.
///
/// Das Mitlesen ist Pflicht, nicht Kür: Wer stderr auf `piped()` setzt und
/// stderr wegwirft, meldet dem Anwender fälschlich eine laufende Sitzung.
/// Das Protokoll muss dabei fortlaufend gelesen werden: läuft der Pipe-Puffer
/// voll, blockiert FreeRDP beim Schreiben und das Sitzungsfenster friert ein.
///
/// Die Datei kam hinzu, weil ein Abbruch **nach** dem Start bis dahin
/// unerklärbar war: `early_failure` prüft nur 600 ms, danach landeten alle
/// Meldungen in einem Ringpuffer, den nichts mehr auswertete. Zwei gemeldete
/// Abbrüche (nach 141 s und 44 s) liessen sich deshalb nicht aufklären.
fn drain_stderr(child: &mut std::process::Child, log_file: Option<PathBuf>) -> Arc<Mutex<Vec<String>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let Some(stderr) = child.stderr.take() else {
        return log;
    };
    let sink = Arc::clone(&log);
    std::thread::spawn(move || {
        let mut file = log_file.and_then(|path| {
            path.parent().map(|dir| fs::create_dir_all(dir));
            rotate_logs(&path);
            let mut handle = fs::File::create(path).ok()?;
            // Ohne Startzeitpunkt lässt sich eine Zeile im Sitzungsprotokoll
            // nicht mit dem Systemprotokoll zusammenführen -- FreeRDPs WLog
            // stempelt keine Uhrzeit. Genau daran scheiterte die Auswertung
            // der ersten Abbruchmeldungen.
            let _ = writeln!(handle, "=== Sitzung gestartet: {} ===", local_timestamp());
            Some(handle)
        });
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Some(handle) = file.as_mut() {
                let _ = writeln!(handle, "{line}");
            }
            let Ok(mut entries) = sink.lock() else { return };
            // Nur die jüngsten Meldungen behalten, damit der Speicher bei
            // langen Sitzungen nicht unbegrenzt wächst.
            if entries.len() == 200 {
                entries.remove(0);
            }
            entries.push(line);
        }
    });
    log
}

/// Wie viele abgeschlossene Sitzungen je Profil aufgehoben werden.
const LOG_GENERATIONS: usize = 5;

/// Schiebt `x.log` nach `x.log.1`, `x.log.1` nach `x.log.2` und so fort.
///
/// Vorher wurde beim Start schlicht überschrieben. Das kostete den Beleg für
/// fünf gemeldete Abbrüche: Der Anwender verbindet nach einem Abbruch sofort
/// neu -- und löschte damit genau das Protokoll, das den Grund enthielt.
/// Fehlschläge sind bewusst folgenlos; ein Protokoll ist kein Grund, eine
/// Verbindung zu verweigern.
fn rotate_logs(path: &std::path::Path) {
    if !path.exists() {
        return;
    }
    // Nicht `with_extension`: enthielte der Dateiname einen Punkt, ersetzte
    // das den falschen Teil. Angehängt wird stattdessen am ganzen Namen.
    let generation = |n: usize| {
        let mut name = path.as_os_str().to_os_string();
        name.push(format!(".{n}"));
        PathBuf::from(name)
    };
    let _ = fs::remove_file(generation(LOG_GENERATIONS));
    for n in (1..LOG_GENERATIONS).rev() {
        let _ = fs::rename(generation(n), generation(n + 1));
    }
    let _ = fs::rename(path, generation(1));
}

/// Ortszeit als lesbarer Zeitstempel.
///
/// Bewusst über `/bin/date` statt über eine Datumsbibliothek: Die Zeitzone
/// stammt so vom System und muss nicht nachgebildet werden, und es kommt keine
/// weitere Abhängigkeit hinzu. Schlägt der Aufruf fehl, bleibt die
/// Unix-Sekunde -- unschön, aber immer noch vergleichbar.
fn local_timestamp() -> String {
    let formatted = Command::new("/bin/date")
        .arg("+%Y-%m-%d %H:%M:%S %z")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|text| !text.is_empty());
    formatted.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("{} (Unix-Sekunde)", d.as_secs()))
            .unwrap_or_else(|_| "unbekannt".into())
    })
}

fn failure_reason(log: &Arc<Mutex<Vec<String>>>, status: std::process::ExitStatus) -> String {
    let detail = log.lock().ok().and_then(|entries| {
        entries
            .iter()
            .filter(|line| line.contains("[ERROR]"))
            .next_back()
            .map(|line| line.rsplit("] - ").next().unwrap_or(line).trim().to_string())
            .filter(|line| !line.is_empty())
    });
    match detail {
        Some(message) => err1("err.freerdpExited", message),
        None => err1("err.freerdpExitedUnexpectedly", status),
    }
}

fn early_failure(child: &mut std::process::Child, log: &Arc<Mutex<Vec<String>>>) -> Option<String> {
    std::thread::sleep(Duration::from_millis(600));
    match child.try_wait() {
        Ok(Some(status)) if !status.success() => Some(failure_reason(log, status)),
        _ => None,
    }
}

/// Erzeugt ein schlankes Startbundle, dessen Ordnername der Profilname ist.
///
/// Grund: Das Dock beschriftet ein Symbol mit dem **Ordnernamen** des Bundles –
/// gemessen, nicht vermutet. `CFBundleName`, `CFBundleDisplayName` und
/// `CFBundleExecutable` blieben in drei Versuchen wirkungslos. Zusätzlich muss
/// die Bundle-Kennung je Profil verschieden sein: LaunchServices merkt sich den
/// Namen **je Kennung**, sonst gewinnt ein alter Eintrag den Ordnernamen.
///
/// Kopiert wird nichts: `MacOS`, `Resources` und `Frameworks` sind Verweise auf
/// das echte Bundle. Ein Startbundle kostet dadurch rund 4 KB statt 44 MB, und
/// die ausführbare Datei bleibt die signierte im Programmordner.
///
/// Schlägt irgendetwas fehl, gibt es `None` – dann startet die Sitzung wie
/// bisher direkt. Ein hübscher Name ist kein Grund, eine Verbindung zu verlieren.
#[cfg(target_os = "macos")]
fn session_launcher(profile: &RemoteProfile, binary: &Path) -> Option<PathBuf> {
    let contents = binary.parent()?.parent()?;
    // Nur für die mitgelieferten Bundles; ein Homebrew-Binary hat kein Contents.
    if contents.file_name()? != "Contents" {
        return None;
    }
    let name = window_title(profile);
    // Je Profil ein eigener Ordner, damit gleichnamige Profile sich nicht
    // überschreiben.
    let root = config_dir().ok()?.join("sessions").join(safe_component(&profile.id));
    let folder = format!("{}.app", safe_component(&name));
    fs::create_dir_all(&root).ok()?;
    // Nach einer Umbenennung bliebe sonst das Bundle mit dem alten Namen
    // liegen. Das gesuchte darf dabei nicht mitgelöscht werden: Eine zweite
    // Sitzung würde sonst der ersten das Bundle unter den Füssen wegziehen –
    // gemessen, der laufende Prozess verschwand.
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if entry.file_name() != folder.as_str() {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
    let app_contents = root.join(&folder).join("Contents");
    fs::create_dir_all(&app_contents).ok()?;
    for part in ["MacOS", "Resources", "Frameworks"] {
        let source = contents.join(part);
        let link = app_contents.join(part);
        if source.exists() && !link.exists() {
            std::os::unix::fs::symlink(&source, link).ok()?;
        }
    }
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>MacFreeRDP</string>
  <key>CFBundleIdentifier</key><string>com.remotedesk.session.{id}</string>
  <key>CFBundleName</key><string>{name}</string>
  <key>CFBundleDisplayName</key><string>{name}</string>
  <key>CFBundleIconFile</key><string>RemoteDeskRDP</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSMainNibFile</key><string>MainMenu</string>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
"#,
        id = safe_identifier(&profile.id),
        name = xml_escape(&name),
    );
    fs::write(app_contents.join("Info.plist"), plist).ok()?;
    let started = app_contents.join("MacOS").join(binary.file_name()?);
    started.exists().then_some(started)
}

/// Macht aus beliebigem Text einen unbedenklichen Ordner- oder Dateinamen.
///
/// `/` trennt Pfade, ein führender Punkt versteckt den Ordner, `:` zeigt der
/// Finder als `/` an. Der Name kommt aus einem Profil und damit vom Nutzer.
/// Wird sowohl für die Startbundles als auch für die Sitzungsprotokolle
/// gebraucht, deshalb plattformunabhängig.
fn safe_component(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c == '/' || c == ':' || c.is_control() { '_' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.').trim().to_string();
    if cleaned.is_empty() { "Sitzung".into() } else { cleaned }
}

/// Bundle-Kennungen dürfen nur Buchstaben, Ziffern, Punkt und Bindestrich
/// enthalten. Die Profil-ID ist eine UUID, aber verlassen wollen wir uns nicht
/// darauf – sie stammt aus einer Datei, die auch von Hand entstehen kann.
#[cfg(target_os = "macos")]
fn safe_identifier(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    if cleaned.is_empty() { "unbenannt".into() } else { cleaned }
}

/// Der Profilname landet in einer XML-Datei; drei Zeichen müssen maskiert
/// werden, sonst entsteht kaputtes Plist und das Startbundle taugt nichts.
#[cfg(target_os = "macos")]
fn xml_escape(raw: &str) -> String {
    raw.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Baut die Verbindung auf. Vorgeschaltet ist die Prüfung auf eine bereits
/// laufende Sitzung desselben Profils – siehe `Sessions`.
fn connect_rdp(app: &tauri::AppHandle, profile: &RemoteProfile) -> Result<(), String> {
    use tauri::Manager;
    if app.state::<Sessions>().claim(&profile.id).is_err() {
        // Kein Fehler, sondern der Normalfall bei einem zweiten Klick: Der
        // Anwender will zu *dieser* Sitzung. Also holen wir sie nach vorne,
        // statt eine zweite aufzubauen, die der Server sofort quittiert,
        // indem er die erste trennt.
        focus_running_session(&profile.id);
        return Ok(());
    }
    let outcome = spawn_session(app, profile);
    if outcome.is_err() {
        // Anmeldung zurücknehmen, sonst bliebe das Profil dauerhaft gesperrt.
        app.state::<Sessions>().release(&profile.id);
    }
    outcome
}

/// Verbindungsziel für den lokalen RFB-Proxy. IPv6 braucht Klammern, damit
/// Adresse und Port nicht verwechselt werden. Der Host wird später direkt an
/// `TcpStream::connect` übergeben und wird nie als Prozessargument verwendet.
fn vnc_target(profile: &RemoteProfile) -> String {
    let host = profile.host.trim();
    let host = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    format!("{host}:{}", profile.vnc_port)
}

/// Ein lokaler TCP-Port allein schützt nicht vor anderen Prozessen desselben
/// Benutzerkontos. Der Viewer erhält daher pro Sitzung einen nicht erratbaren
/// Einmal-Token; ungültige WebSocket-Anfragen werden abgewiesen, bis der
/// rechtmäßige Viewer innerhalb des kurzen Zeitfensters verbunden ist.
fn vnc_session_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .map_err(|error| err1("err.vncProxyStart", error))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Startet den eingebetteten, universellen noVNC-Viewer über einen nur lokal
/// erreichbaren RFB-zu-WebSocket-Proxy. Die RFB-Kodierung wird zwischen noVNC
/// und Server ausgehandelt; deshalb gibt es keinen TightVNC-Sonderweg und
/// keine fest erzwungene Kompression.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VncEndpoint { websocket_url: String }

#[tauri::command]
async fn start_vnc_session(app: tauri::AppHandle, profile: RemoteProfile) -> Result<VncEndpoint, String> {
    use tauri::Manager;
    if profile.protocol != Protocol::Vnc { return Err("err.vncProfileRequired".into()); }
    validate_profile(&profile)?;
    let token = vnc_session_token()?;
    if app.state::<Sessions>().claim(&profile.id).is_err() { return Err("err.vncAlreadyRunning".into()); }
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => { app.state::<Sessions>().release(&profile.id); return Err(err1("err.vncProxyStart", error)); }
    };
    let port = match listener.local_addr() {
        Ok(address) => address.port(),
        Err(error) => {
            app.state::<Sessions>().release(&profile.id);
            return Err(err1("err.vncProxyStart", error));
        }
    };
    let target = vnc_target(&profile);
    let expected_path = format!("/?token={token}");
    let id = profile.id.clone(); let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            let Ok(Ok((socket, _))) = tokio::time::timeout(remaining, listener.accept()).await else { break; };
            let path = expected_path.clone();
            let websocket = tokio_tungstenite::accept_hdr_async(socket, move |
                request: &tokio_tungstenite::tungstenite::handshake::server::Request,
                response: tokio_tungstenite::tungstenite::handshake::server::Response,
            | {
                if request.uri().path_and_query().map(|value| value.as_str()) == Some(path.as_str()) {
                    Ok(response)
                } else {
                    Err(tokio_tungstenite::tungstenite::http::Response::builder()
                        .status(403)
                        .body(Some("Invalid VNC session token".to_string()))
                        .expect("valid static HTTP response"))
                }
            }).await;
            let Ok(websocket) = websocket else { continue; };
            if let Ok(remote) = tokio::net::TcpStream::connect(&target).await {
                let (mut sink, mut stream) = websocket.split();
                let (mut remote_read, mut remote_write) = remote.into_split();
                let web_to_rfb = async { while let Some(message) = stream.next().await { match message { Ok(tokio_tungstenite::tungstenite::Message::Binary(data)) => { if tokio::io::AsyncWriteExt::write_all(&mut remote_write, &data).await.is_err() { break; } }, Ok(tokio_tungstenite::tungstenite::Message::Close(_)) | Err(_) => break, _ => {} } } };
                let rfb_to_web = async { let mut buffer = [0u8; 64 * 1024]; loop { let read = match tokio::io::AsyncReadExt::read(&mut remote_read, &mut buffer).await { Ok(0) | Err(_) => break, Ok(read) => read }; if sink.send(tokio_tungstenite::tungstenite::Message::Binary(buffer[..read].to_vec().into())).await.is_err() { break; } } };
                tokio::select! { _ = web_to_rfb => {}, _ = rfb_to_web => {} }
                break;
            }
            break;
        }
        handle.state::<Sessions>().release(&id);
    });
    Ok(VncEndpoint { websocket_url: format!("ws://127.0.0.1:{port}/?token={token}") })
}

fn spawn_session(app: &tauri::AppHandle, profile: &RemoteProfile) -> Result<(), String> {
    use tauri::Manager;
    let binary = rdp_executable(app)?;
    let backend = backend_kind(&binary);
    if backend == Backend::X11 {
        Command::new("/usr/bin/open")
            .args(["-a", "XQuartz"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| err1("err.xquartz", e))?;
        std::thread::sleep(Duration::from_millis(500));
    }
    let arguments = rdp_arguments(profile, backend)?;
    // Startbundle mit dem Profilnamen, damit das Dock die Sitzung benennt.
    // Nur eine Verschönerung; misslingt sie, bleibt es beim echten Pfad.
    #[cfg(target_os = "macos")]
    let binary = session_launcher(profile, &binary).unwrap_or(binary);
    // FreeRDP liest die einzelnen Optionen aus stdin. Dadurch taucht ein
    // gegebenenfalls gespeichertes Kennwort nicht in der Prozessliste auf.
    let mut command = Command::new(binary);
    command
        .arg("/args-from:stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    match backend {
        Backend::X11 => {
            command.env("DISPLAY", ":0");
        }
        // Der Metal-Renderer von SDL bleibt unter macOS in
        // `[CAMetalLayer nextDrawable]` hängen; das Fenster friert dann ein.
        Backend::Sdl => {
            command.env("SDL_RENDER_DRIVER", "opengl");
        }
        Backend::Cocoa => {}
    }
    let mut child = command
        .spawn()
        .map_err(|e| err1("err.freerdpStart", e))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "FreeRDP-Eingabe konnte nicht geöffnet werden".to_string())?;
        for argument in arguments {
            writeln!(stdin, "{argument}").map_err(|e| err1("err.freerdpInput", e))?;
        }
    }
    // FreeRDP wertet die Argumente erst nach dem Dateiende aus.
    drop(child.stdin.take());
    // Muss vor der ersten Wartezeit laufen, sonst blockiert FreeRDP an einem
    // vollen stderr-Puffer und das Sitzungsfenster reagiert nicht mehr.
    let log = drain_stderr(&mut child, session_log_path(&profile.id));
    if let Some(message) = early_failure(&mut child, &log) {
        return Err(message);
    }
    let id = profile.id.clone();
    app.state::<Sessions>().attach(&id, child.id());
    // Verhindert einen Zombie-Prozess, sobald die Sitzung beendet wird, und
    // gibt das Profil für einen neuen Verbindungsversuch wieder frei.
    let handle = app.clone();
    std::thread::spawn(move || {
        let status = child.wait();
        // Warum die Sitzung endete, gehört ans Ende des Protokolls – sonst
        // steht dort nur abgeschnittener Verkehr und niemand weiss, ob
        // FreeRDP abstürzte, sich beendete oder getrennt wurde.
        //
        // Bewusst **vor** der Freigabe: sonst könnte ein sofortiger Neustart
        // die Datei bereits weggeschoben haben und der Grund landete am Ende
        // der frisch begonnenen Sitzung.
        if let Some(path) = session_log_path(&id) {
            if let Ok(mut file) = fs::OpenOptions::new().append(true).open(path) {
                let now = local_timestamp();
                let _ = match status {
                    Ok(state) => writeln!(file, "--- Sitzung beendet {now}: {state} ---"),
                    Err(e) => writeln!(file, "--- Sitzung beendet {now}, Status unbekannt: {e} ---"),
                };
            }
        }
        handle.state::<Sessions>().release(&id);
    });
    Ok(())
}

/// Ablageort des Sitzungsprotokolls. `None`, wenn kein Ordner ermittelbar ist –
/// dann läuft die Sitzung ohne Protokoll weiter, statt gar nicht zu starten.
fn session_log_path(profile_id: &str) -> Option<PathBuf> {
    Some(
        config_dir()
            .ok()?
            .join("logs")
            .join(format!("{}.log", safe_component(profile_id))),
    )
}

/// Ein einheitlicher Eintrag für S3-Buckets, Swift-Container sowie deren
/// Objekte und Präfixe. Die UI benötigt damit keine Protokollkenntnis.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ObjectEntry {
    name: String,
    is_prefix: bool,
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ObjectListing {
    container: String,
    prefix: String,
    entries: Vec<ObjectEntry>,
}

#[derive(Deserialize)]
struct S3ListResult {
    #[serde(rename = "Contents", default)]
    contents: Vec<S3Content>,
    #[serde(rename = "CommonPrefixes", default)]
    prefixes: Vec<S3Prefix>,
}

#[derive(Deserialize)]
struct S3Content {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Size")]
    size: u64,
    #[serde(rename = "LastModified")]
    modified: Option<String>,
}

#[derive(Deserialize)]
struct S3Prefix {
    #[serde(rename = "Prefix")]
    prefix: String,
}

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC akzeptiert Schlüssel jeder Länge");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// AWS erwartet RFC-3986-Kodierung; `application/x-www-form-urlencoded`
/// wäre falsch, weil es Leerzeichen als `+` darstellt.
fn aws_encode(value: &str, keep_slash: bool) -> String {
    value.bytes().flat_map(|byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') || (keep_slash && byte == b'/') {
            String::from(byte as char).chars().collect::<Vec<_>>()
        } else {
            format!("%{byte:02X}").chars().collect()
        }
    }).collect()
}

fn object_http_client() -> Result<HttpClient, String> {
    HttpClient::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|error| err1("err.objectRequest", error))
}

fn object_response_error(response: reqwest::blocking::Response) -> String {
    let status = response.status();
    let detail = response.text().unwrap_or_default();
    // Antworten können XML oder HTML enthalten; eine kurze, zeilenfreie
    // Vorschau hilft bei Endpoint-Fehlern, ohne Zugangsdaten preiszugeben.
    let preview = detail.replace(['\r', '\n'], " ").chars().take(240).collect::<String>();
    err1("err.objectRequest", if preview.is_empty() { status.to_string() } else { format!("{status}: {preview}") })
}

fn require_object_secret(profile: &RemoteProfile) -> Result<String, String> {
    object_secret(&profile.id)?.filter(|secret| !secret.is_empty()).ok_or_else(|| "err.objectSecretMissing".into())
}

fn valid_container(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.len() > 255 || name.contains(['/', '\\', '\r', '\n']) {
        return Err("err.objectContainerInvalid".into());
    }
    Ok(())
}

fn s3_url(profile: &RemoteProfile, container: &str, key: &str, query: Option<&str>) -> Result<reqwest::Url, String> {
    valid_container(container)?;
    let mut url = reqwest::Url::parse(profile.object_endpoint.trim()).map_err(|_| "err.objectEndpointInvalid")?;
    let base_path = url.path().trim_end_matches('/').to_string();
    let encoded_key = aws_encode(key.trim_start_matches('/'), true);
    if profile.object_path_style {
        url.set_path(&format!("{base_path}/{}/{}", aws_encode(container, false), encoded_key));
    } else {
        let host = url.host_str().ok_or("err.objectEndpointInvalid")?;
        url.set_host(Some(&format!("{}.{}", aws_encode(container, false), host))).map_err(|_| "err.objectEndpointInvalid")?;
        url.set_path(&format!("{base_path}/{encoded_key}"));
    }
    url.set_query(query);
    Ok(url)
}

fn s3_request(profile: &RemoteProfile, method: reqwest::Method, url: reqwest::Url, payload: &[u8]) -> Result<RequestBuilder, String> {
    let secret = require_object_secret(profile)?;
    let now = Utc::now();
    let short_date = now.format("%Y%m%d").to_string();
    let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();
    let payload_hash = sha256_hex(payload);
    let host = match url.port() {
        Some(port) => format!("{}:{port}", url.host_str().ok_or("err.objectEndpointInvalid")?),
        None => url.host_str().ok_or("err.objectEndpointInvalid")?.to_string(),
    };
    let canonical_headers = format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{timestamp}\n");
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}", method.as_str(), url.path(), url.query().unwrap_or(""), canonical_headers, signed_headers, payload_hash);
    let scope = format!("{short_date}/{}/s3/aws4_request", profile.object_region.trim());
    let string_to_sign = format!("AWS4-HMAC-SHA256\n{timestamp}\n{scope}\n{}", sha256_hex(canonical_request.as_bytes()));
    let date_key = hmac_sha256(format!("AWS4{secret}").as_bytes(), &short_date);
    let region_key = hmac_sha256(&date_key, profile.object_region.trim());
    let service_key = hmac_sha256(&region_key, "s3");
    let signing_key = hmac_sha256(&service_key, "aws4_request");
    let signature = hex::encode(hmac_sha256(&signing_key, &string_to_sign));
    let authorization = format!("AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}", profile.object_access_key.trim());
    Ok(object_http_client()?.request(method, url)
        .header("host", host)
        .header("x-amz-content-sha256", payload_hash)
        .header("x-amz-date", timestamp)
        .header("authorization", authorization))
}

fn s3_list(profile: &RemoteProfile, container: &str, prefix: &str) -> Result<ObjectListing, String> {
    let query = format!("delimiter=%2F&list-type=2&prefix={}", aws_encode(prefix, false));
    let response = s3_request(profile, reqwest::Method::GET, s3_url(profile, container, "", Some(&query))?, &[])?.send().map_err(|error| err1("err.objectRequest", error))?;
    if !response.status().is_success() { return Err(object_response_error(response)); }
    let result: S3ListResult = xml_from_str(&response.text().map_err(|error| err1("err.objectRequest", error))?).map_err(|error| err1("err.objectRequest", error))?;
    let mut entries = result.prefixes.into_iter().map(|item| ObjectEntry { name: item.prefix, is_prefix: true, size: None, modified: None }).collect::<Vec<_>>();
    entries.extend(result.contents.into_iter().filter(|item| item.key != prefix).map(|item| ObjectEntry { name: item.key, is_prefix: false, size: Some(item.size), modified: item.modified }));
    entries.sort_by(|left, right| right.is_prefix.cmp(&left.is_prefix).then_with(|| left.name.cmp(&right.name)));
    Ok(ObjectListing { container: container.into(), prefix: prefix.into(), entries })
}

fn s3_list_containers(profile: &RemoteProfile) -> Result<ObjectListing, String> {
    let mut url = reqwest::Url::parse(profile.object_endpoint.trim()).map_err(|_| "err.objectEndpointInvalid")?;
    url.set_query(None);
    let response = s3_request(profile, reqwest::Method::GET, url, &[])?.send().map_err(|error| err1("err.objectRequest", error))?;
    if !response.status().is_success() { return Err(object_response_error(response)); }
    #[derive(Deserialize)] struct Buckets { #[serde(rename = "Bucket", default)] bucket: Vec<Bucket> }
    #[derive(Deserialize)] struct Bucket { #[serde(rename = "Name")] name: String }
    #[derive(Deserialize)] struct ResultXml { #[serde(rename = "Buckets")] buckets: Buckets }
    let result: ResultXml = xml_from_str(&response.text().map_err(|error| err1("err.objectRequest", error))?).map_err(|error| err1("err.objectRequest", error))?;
    Ok(ObjectListing { container: String::new(), prefix: String::new(), entries: result.buckets.bucket.into_iter().map(|bucket| ObjectEntry { name: bucket.name, is_prefix: true, size: None, modified: None }).collect() })
}

#[derive(Clone)]
struct SwiftSession { storage_url: String, token: String }

/// Standort und Keystone-Pfad sind getrennte Profilfelder. Falls ein älteres
/// Profil den Pfad bereits in der Standort-URL enthält, erkennen wir das und
/// hängen ihn nicht doppelt an.
fn swift_auth_url(endpoint: &str, identity_path: &str, version: SwiftAuthVersion) -> Result<reqwest::Url, String> {
    let mut endpoint = reqwest::Url::parse(endpoint.trim()).map_err(|_| "err.objectEndpointInvalid")?;
    let base_path = endpoint.path().trim_end_matches('/');
    let identity_path = identity_path.trim_end_matches('/');
    let path = if base_path.ends_with(identity_path) { base_path.to_string() } else { format!("{base_path}{identity_path}") };
    let suffix = match version { SwiftAuthVersion::V3 => "auth/tokens", SwiftAuthVersion::V2 => "tokens" };
    endpoint.set_path(&format!("{path}/{suffix}"));
    Ok(endpoint)
}

fn swift_session(profile: &RemoteProfile) -> Result<SwiftSession, String> {
    let secret = require_object_secret(profile)?;
    let endpoint = swift_auth_url(&profile.object_endpoint, &profile.swift_identity_path, profile.swift_auth_version)?;
    let user_domain = if profile.swift_user_domain.trim().is_empty() { "Default" } else { profile.swift_user_domain.trim() };
    let project_domain = if profile.swift_project_domain.trim().is_empty() { "Default" } else { profile.swift_project_domain.trim() };
    let request = match profile.swift_auth_version {
        SwiftAuthVersion::V3 => serde_json::json!({
            "auth": { "identity": { "methods": ["password"], "password": { "user": {
                "name": profile.username.trim(), "domain": { "name": user_domain }, "password": secret
            }}}, "scope": { "project": { "name": profile.swift_project.trim(), "domain": { "name": project_domain } }}}
        }),
        SwiftAuthVersion::V2 => serde_json::json!({
            "auth": { "passwordCredentials": { "username": profile.username.trim(), "password": secret }, "tenantName": profile.swift_project.trim() }
        }),
    };
    let response = object_http_client()?.post(endpoint).json(&request).send().map_err(|error| err1("err.objectRequest", error))?;
    if !response.status().is_success() { return Err(object_response_error(response)); }
    let v3_token = response.headers().get("X-Subject-Token").and_then(|value| value.to_str().ok()).filter(|value| !value.is_empty()).map(str::to_owned);
    let body: serde_json::Value = response.json().map_err(|error| err1("err.objectRequest", error))?;
    let (token, selected) = match profile.swift_auth_version {
        SwiftAuthVersion::V3 => {
            let token = v3_token.ok_or("err.swiftTokenMissing")?;
            let endpoints = body.pointer("/token/catalog").and_then(|catalog| catalog.as_array()).into_iter().flatten()
                .filter(|service| service.get("type").and_then(|value| value.as_str()) == Some("object-store"))
                .flat_map(|service| service.get("endpoints").and_then(|value| value.as_array()).into_iter().flatten());
            let selected = endpoints.filter(|endpoint| {
                endpoint.get("interface").and_then(|value| value.as_str()) == Some("public")
                    && (profile.object_region.trim().is_empty() || endpoint.get("region").and_then(|value| value.as_str()) == Some(profile.object_region.trim()))
            }).find_map(|endpoint| endpoint.get("url").and_then(|value| value.as_str())).ok_or("err.swiftStorageEndpointMissing")?;
            (token, selected)
        }
        SwiftAuthVersion::V2 => {
            let token = body.pointer("/access/token/id").and_then(|value| value.as_str()).filter(|value| !value.is_empty()).ok_or("err.swiftTokenMissing")?.to_string();
            let endpoints = body.pointer("/access/serviceCatalog").and_then(|catalog| catalog.as_array()).into_iter().flatten()
                .filter(|service| service.get("type").and_then(|value| value.as_str()) == Some("object-store"))
                .flat_map(|service| service.get("endpoints").and_then(|value| value.as_array()).into_iter().flatten());
            let selected = endpoints.filter(|endpoint| profile.object_region.trim().is_empty() || endpoint.get("region").and_then(|value| value.as_str()) == Some(profile.object_region.trim()))
                .find_map(|endpoint| endpoint.get("publicURL").and_then(|value| value.as_str())).ok_or("err.swiftStorageEndpointMissing")?;
            (token, selected)
        }
    };
    Ok(SwiftSession { storage_url: selected.trim_end_matches('/').to_string(), token })
}

fn swift_url(session: &SwiftSession, container: &str, key: &str, query: Option<&str>) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(&session.storage_url).map_err(|_| "err.objectEndpointInvalid")?;
    if !container.trim().is_empty() {
        valid_container(container)?;
        let path = format!("{}/{}/{}", url.path().trim_end_matches('/'), aws_encode(container, false), aws_encode(key.trim_start_matches('/'), true));
        url.set_path(&path);
    }
    url.set_query(query);
    Ok(url)
}

fn swift_request(client: &HttpClient, session: &SwiftSession, method: reqwest::Method, url: reqwest::Url) -> RequestBuilder {
    client.request(method, url).header("X-Auth-Token", &session.token)
}

fn swift_list(profile: &RemoteProfile, container: &str, prefix: &str) -> Result<ObjectListing, String> {
    let client = object_http_client()?;
    let session = swift_session(profile)?;
    let query = format!("format=json&delimiter=%2F&prefix={}", aws_encode(prefix, false));
    let response = swift_request(&client, &session, reqwest::Method::GET, swift_url(&session, container, "", Some(&query))?).send().map_err(|error| err1("err.objectRequest", error))?;
    if !response.status().is_success() { return Err(object_response_error(response)); }
    let items: Vec<serde_json::Value> = response.json().map_err(|error| err1("err.objectRequest", error))?;
    let entries = items.into_iter().filter_map(|item| {
        if let Some(prefix) = item.get("subdir").and_then(|value| value.as_str()) { Some(ObjectEntry { name: prefix.into(), is_prefix: true, size: None, modified: None }) }
        else { item.get("name").and_then(|value| value.as_str()).map(|name| ObjectEntry { name: name.into(), is_prefix: false, size: item.get("bytes").and_then(|value| value.as_u64()), modified: item.get("last_modified").and_then(|value| value.as_str()).map(str::to_owned) }) }
    }).collect();
    Ok(ObjectListing { container: container.into(), prefix: prefix.into(), entries })
}

fn swift_list_containers(profile: &RemoteProfile) -> Result<ObjectListing, String> {
    let client = object_http_client()?;
    let session = swift_session(profile)?;
    let response = swift_request(&client, &session, reqwest::Method::GET, swift_url(&session, "", "", Some("format=json"))?).send().map_err(|error| err1("err.objectRequest", error))?;
    if !response.status().is_success() { return Err(object_response_error(response)); }
    let items: Vec<serde_json::Value> = response.json().map_err(|error| err1("err.objectRequest", error))?;
    Ok(ObjectListing { container: String::new(), prefix: String::new(), entries: items.into_iter().filter_map(|item| item.get("name").and_then(|value| value.as_str()).map(|name| ObjectEntry { name: name.into(), is_prefix: true, size: item.get("bytes").and_then(|value| value.as_u64()), modified: None })).collect() })
}

#[tauri::command]
fn list_object_storage(profile: RemoteProfile, container: String, prefix: String) -> Result<ObjectListing, String> {
    validate_profile(&profile)?;
    if !matches!(profile.protocol, Protocol::S3 | Protocol::Swift) { return Err("err.objectProfileRequired".into()); }
    if container.trim().is_empty() {
        return match profile.protocol { Protocol::S3 => s3_list_containers(&profile), Protocol::Swift => swift_list_containers(&profile), _ => unreachable!() };
    }
    match profile.protocol { Protocol::S3 => s3_list(&profile, &container, &prefix), Protocol::Swift => swift_list(&profile, &container, &prefix), _ => unreachable!() }
}

fn s3_put(profile: &RemoteProfile, container: &str, key: &str, bytes: Vec<u8>) -> Result<(), String> {
    let response = s3_request(profile, reqwest::Method::PUT, s3_url(profile, container, key, None)?, &bytes)?.header(CONTENT_TYPE, "application/octet-stream").body(bytes).send().map_err(|error| err1("err.objectRequest", error))?;
    if response.status().is_success() { Ok(()) } else { Err(object_response_error(response)) }
}

fn swift_put(profile: &RemoteProfile, container: &str, key: &str, bytes: Vec<u8>) -> Result<(), String> {
    let client = object_http_client()?; let session = swift_session(profile)?;
    let response = swift_request(&client, &session, reqwest::Method::PUT, swift_url(&session, container, key, None)?).header(CONTENT_TYPE, "application/octet-stream").body(bytes).send().map_err(|error| err1("err.objectRequest", error))?;
    if response.status().is_success() { Ok(()) } else { Err(object_response_error(response)) }
}

#[tauri::command]
fn upload_object(profile: RemoteProfile, container: String, key: String, source_path: String) -> Result<(), String> {
    validate_profile(&profile)?; valid_container(&container)?;
    if key.trim().is_empty() || key.starts_with('/') || key.contains(['\r', '\n']) { return Err("err.objectKeyInvalid".into()); }
    let bytes = fs::read(&source_path).map_err(|error| err1("err.objectFileRead", error))?;
    match profile.protocol { Protocol::S3 => s3_put(&profile, &container, &key, bytes), Protocol::Swift => swift_put(&profile, &container, &key, bytes), _ => Err("err.objectProfileRequired".into()) }
}

#[tauri::command]
fn delete_object(profile: RemoteProfile, container: String, key: String) -> Result<(), String> {
    validate_profile(&profile)?; valid_container(&container)?;
    let response = match profile.protocol {
        Protocol::S3 => s3_request(&profile, reqwest::Method::DELETE, s3_url(&profile, &container, &key, None)?, &[])?.send().map_err(|error| err1("err.objectRequest", error))?,
        Protocol::Swift => { let client = object_http_client()?; let session = swift_session(&profile)?; swift_request(&client, &session, reqwest::Method::DELETE, swift_url(&session, &container, &key, None)?).send().map_err(|error| err1("err.objectRequest", error))? },
        _ => return Err("err.objectProfileRequired".into()),
    };
    if response.status().is_success() { Ok(()) } else { Err(object_response_error(response)) }
}

#[tauri::command]
fn download_object(profile: RemoteProfile, container: String, key: String, destination_directory: String) -> Result<String, String> {
    validate_profile(&profile)?; valid_container(&container)?;
    let filename = Path::new(&key).file_name().and_then(|name| name.to_str()).filter(|name| !name.is_empty()).ok_or("err.objectKeyInvalid")?;
    let destination = Path::new(&destination_directory).join(filename);
    if destination.exists() { return Err("err.objectDownloadExists".into()); }
    let response = match profile.protocol {
        Protocol::S3 => s3_request(&profile, reqwest::Method::GET, s3_url(&profile, &container, &key, None)?, &[])?.send().map_err(|error| err1("err.objectRequest", error))?,
        Protocol::Swift => { let client = object_http_client()?; let session = swift_session(&profile)?; swift_request(&client, &session, reqwest::Method::GET, swift_url(&session, &container, &key, None)?).send().map_err(|error| err1("err.objectRequest", error))? },
        _ => return Err("err.objectProfileRequired".into()),
    };
    if !response.status().is_success() { return Err(object_response_error(response)); }
    let bytes = response.bytes().map_err(|error| err1("err.objectRequest", error))?;
    fs::write(&destination, bytes).map_err(|error| err1("err.objectFileWrite", error))?;
    Ok(destination.display().to_string())
}

#[tauri::command]
fn connect_profile(app: tauri::AppHandle, profile: RemoteProfile) -> Result<(), String> {
    validate_profile(&profile)?;
    match profile.protocol {
        Protocol::Rdp => connect_rdp(&app, &profile),
        Protocol::Vnc => Err("err.vncEmbeddedOnly".into()),
        Protocol::Ssh => Err("err.sshEmbeddedOnly".into()),
        Protocol::Sftp => Err("err.sftpEmbeddedOnly".into()),
        Protocol::Mosh => Err("err.moshEmbeddedOnly".into()),
        Protocol::S3 | Protocol::Swift => Err("err.objectEmbeddedOnly".into()),
    }
}

/// Liest die Profil-ID aus `remotedesk://connect?id=<uuid>`.
///
/// Bewusst eng gefasst: falsches Schema, falsche Aktion oder fehlende ID
/// ergeben `None`. Ein unbekannter Aufruf soll nichts tun statt zu raten.
fn profile_id_from_url(url: &tauri::Url) -> Option<String> {
    if url.scheme() != "remotedesk" {
        return None;
    }
    // Bei `remotedesk://connect?...` ist "connect" der Host, nicht der Pfad.
    let action = url.host_str().unwrap_or("");
    if action != "connect" {
        return None;
    }
    let id = url.query_pairs().find(|(key, _)| key == "id")?.1.into_owned();
    let id = id.trim().to_string();
    if id.is_empty() { None } else { Some(id) }
}

/// Laufende Sitzungen, je Profil-ID die Prozesskennung von FreeRDP.
///
/// Zweck: Eine **zweite** Sitzung zum selben Profil ist immer schädlich. Ein
/// RDP-Server lässt je Benutzer nur eine Sitzung zu und trennt die ältere –
/// ohne jede Meldung, das Fenster verschwindet einfach. Genau dieses
/// Fehlerbild wurde gemeldet ("Sitzungen brechen nach etwa einer Minute ab").
///
/// Im Systemprotokoll nachgewiesen: Sitzung 94739 endete 2 Sekunden nach dem
/// Start von 94748; 95072 nach 280 s, als 95744 dazukam; 95888 starb, nachdem
/// eine Messung eine zweite Sitzung öffnete. Um 23:42 lagen vier Starts in
/// 16 Sekunden – drei der Prozesse lebten nur eine Sekunde.
///
/// Ursache war fehlende Rückmeldung: Nach dem Klick in DualBeam passiert
/// sekundenlang sichtbar nichts, also klickt man erneut.
#[derive(Default)]
struct Sessions(Mutex<HashMap<String, u32>>);

struct SshSession {
    id: u64,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    uses_x11: bool,
}

struct SshSessions(Mutex<HashMap<String, SshSession>>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshData { profile_id: String, data: Vec<u8> }

/// Wird nach dem Ende eines Terminal-Prozesses gesendet. Bei einer X11-App
/// kann das Terminalfenster damit geschlossen werden, sobald der per SSH
/// gestartete Prozess beendet ist; ein verbliebenes Dock-Fenster blockiert
/// dann keinen erneuten Verbindungsstart mehr.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshEnded { profile_id: String }

fn ssh_target(profile: &RemoteProfile) -> String {
    if profile.username.trim().is_empty() { profile.host.trim().to_string() }
    else { format!("{}@{}", profile.username.trim(), profile.host.trim()) }
}

/// Mosh wird nicht mitgeliefert: Die optionale lokale Installation bleibt
/// Eigentum des Nutzers und ihre GPL-Lizenz wird nicht in das App-Bundle
/// eingebunden. Homebrew installiert je nach Architektur an einem dieser Orte.
fn mosh_programs() -> Result<(PathBuf, PathBuf), String> {
    for executable in ["/opt/homebrew/bin/mosh", "/usr/local/bin/mosh", "/usr/bin/mosh"] {
        let mosh = PathBuf::from(executable);
        let client = mosh.with_file_name("mosh-client");
        if mosh.is_file() && client.is_file() {
            return Ok((mosh, client));
        }
    }
    Err("err.moshNotInstalled".into())
}

/// Startet den lokalen X-Server bei Bedarf und übernimmt dessen Display in
/// die Finder-gestartete App. Finder vererbt DISPLAY nicht, obwohl XQuartz es
/// in der Benutzer-Launchd-Umgebung bereitstellt; ohne diesen Schritt kann
/// `ssh -X` zwar verbinden, aber keine Linux-Fenster zurück zum Mac leiten.
#[cfg(target_os = "macos")]
fn xquartz_display() -> Result<String, String> {
    let application = ["/Applications/Utilities/XQuartz.app", "/Applications/XQuartz.app"]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.is_dir())
        .ok_or_else(|| "err.xquartzNotInstalled".to_string())?;
    Command::new("/usr/bin/open")
        .arg("-g")
        .arg("-j")
        .arg(application)
        .spawn()
        .map_err(|error| err1("err.xquartzStart", error))?;

    // XQuartz startet nach einer frischen Installation nicht synchron. Wir
    // warten höchstens fünf Sekunden auf sein Launchd-Display statt SSH mit
    // einer leeren DISPLAY-Variable loszuschicken.
    for _ in 0..20 {
        if let Ok(output) = Command::new("/bin/launchctl").args(["getenv", "DISPLAY"]).output() {
            if output.status.success() {
                let display = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !display.is_empty() {
                    return Ok(display);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Err("err.xquartzNotReady".into())
}

/// Beendet XQuartz über seine Bundle-ID. Der Aufruf läuft nur nach einer
/// ausdrücklich aktivierten Profiloption und wenn keine weitere lokale
/// X11-Sitzung dieser App mehr aktiv ist.
#[cfg(target_os = "macos")]
fn stop_xquartz() {
    let _ = Command::new("/usr/bin/osascript")
        .args(["-e", "tell application id \"org.xquartz.X11\" to quit"])
        .spawn();
}

#[cfg(not(target_os = "macos"))]
fn stop_xquartz() {}

#[cfg(not(target_os = "macos"))]
fn xquartz_display() -> Result<String, String> {
    Err("err.xquartzNotInstalled".into())
}

#[tauri::command]
fn start_ssh_session(app: tauri::AppHandle, profile: RemoteProfile, columns: u16, rows: u16) -> Result<(), String> {
    if !matches!(profile.protocol, Protocol::Ssh | Protocol::Sftp | Protocol::Mosh) { return Err("err.terminalProfileRequired".into()); }
    validate_profile(&profile)?;
    if app.state::<SshSessions>().0.lock().unwrap().contains_key(&profile.id) {
        return Err("err.sshAlreadyRunning".into());
    }
    let pty = native_pty_system();
    let pair = pty.openpty(PtySize { rows: rows.max(1), cols: columns.max(1), pixel_width: 0, pixel_height: 0 }).map_err(|e| err1("err.sshStart", e))?;
    let x11_display = if profile.protocol == Protocol::Ssh && profile.x11_forwarding {
        Some(xquartz_display()?)
    } else {
        None
    };
    let mut command = match profile.protocol {
        Protocol::Ssh => {
            let mut command = CommandBuilder::new("/usr/bin/ssh");
            command.arg("-tt");
            if profile.x11_forwarding {
                command.arg("-X");
            }
            command.arg("-p"); command.arg(profile.ssh_port.to_string()); command.arg(ssh_target(&profile));
            if profile.x11_forwarding && !profile.x11_command.trim().is_empty() {
                command.arg(profile.x11_command.trim());
            }
            command
        }
        Protocol::Sftp => {
            let mut command = CommandBuilder::new("/usr/bin/sftp");
            command.arg("-P"); command.arg(profile.ssh_port.to_string()); command.arg(ssh_target(&profile));
            command
        }
        Protocol::Mosh => {
            let (mosh, client) = mosh_programs()?;
            let mut command = CommandBuilder::new(mosh);
            // Der Port wird sicher als u16 gespeichert; damit entstehen keine
            // von Nutzereingaben kontrollierten Shell-Argumente. Der
            // mosh-Starthelfer ist ein Perl-Skript; aus einer Finder-App fehlt
            // /opt/homebrew/bin im PATH. Deshalb bekommt er mosh-client immer
            // absolut übergeben, statt auf dessen PATH-Suche zu vertrauen.
            command.arg(format!("--client={}", client.display()));
            command.arg(format!("--ssh=/usr/bin/ssh -p {}", profile.ssh_port)); command.arg(ssh_target(&profile));
            command
        }
        _ => unreachable!("Nicht-Terminal-Profil wurde vorher abgewiesen"),
    };
    command.env("TERM", profile.ssh_terminal.trim());
    // Aus Finder gestartete macOS-Apps erben häufig nur die POSIX-Standard-
    // Locale (C/US-ASCII). mosh-client lehnt diese explizit ab, weil sein
    // Protokoll UTF-8 voraussetzt. LC_ALL übersteuert zugleich eventuell
    // geerbte LC_CTYPE=C-Werte und macht den Start reproduzierbar.
    command.env("LANG", "en_US.UTF-8");
    command.env("LC_CTYPE", "en_US.UTF-8");
    command.env("LC_ALL", "en_US.UTF-8");
    if let Some(display) = x11_display {
        command.env("DISPLAY", display);
    }
    let mut child = pair.slave.spawn_command(command).map_err(|e| err1("err.sshStart", e))?;
    drop(pair.slave);
    let writer = pair.master.take_writer().map_err(|e| err1("err.sshStart", e))?;
    let mut reader = pair.master.try_clone_reader().map_err(|e| err1("err.sshStart", e))?;
    let session_id = SSH_SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let killer = child.clone_killer();
    let uses_x11 = profile.protocol == Protocol::Ssh && profile.x11_forwarding;
    let close_xquartz_on_exit = uses_x11 && profile.x11_close_xquartz;
    app.state::<SshSessions>().0.lock().unwrap().insert(profile.id.clone(), SshSession { id: session_id, writer, master: pair.master, killer, uses_x11 });
    let id = profile.id.clone(); let output = app.clone(); let state = app.clone();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        while let Ok(count) = reader.read(&mut buffer) {
            if count == 0 { break; }
            let _ = output.emit("ssh-data", SshData { profile_id: id.clone(), data: buffer[..count].to_vec() });
        }
        let _ = child.wait();
        let ssh_sessions = state.state::<SshSessions>();
        let mut sessions = ssh_sessions.0.lock().unwrap();
        let session_ended = if sessions.get(&id).is_some_and(|session| session.id == session_id) {
            sessions.remove(&id);
            true
        } else { false };
        let last_x11_session = session_ended && !sessions.values().any(|session| session.uses_x11);
        drop(sessions);
        // Eine alte Leseschleife darf eine inzwischen gestartete neue Sitzung
        // mit derselben Profil-ID niemals beenden.
        if !session_ended { return; }
        if close_xquartz_on_exit && last_x11_session { stop_xquartz(); }
        let _ = output.emit("ssh-ended", SshEnded { profile_id: id });
    });
    Ok(())
}

#[tauri::command]
fn write_ssh_session(app: tauri::AppHandle, profile_id: String, data: Vec<u8>) -> Result<(), String> {
    let state = app.state::<SshSessions>();
    let mut sessions = state.0.lock().unwrap();
    let session = sessions.get_mut(&profile_id).ok_or("err.sshNotRunning")?;
    session.writer.write_all(&data).map_err(|e| err1("err.sshWrite", e))?;
    session.writer.flush().map_err(|e| err1("err.sshWrite", e))
}

#[tauri::command]
fn resize_ssh_session(app: tauri::AppHandle, profile_id: String, columns: u16, rows: u16) -> Result<(), String> {
    let sessions = app.state::<SshSessions>();
    let sessions = sessions.0.lock().unwrap();
    let session = sessions.get(&profile_id).ok_or("err.sshNotRunning")?;
    session.master.resize(PtySize { rows: rows.max(1), cols: columns.max(1), pixel_width: 0, pixel_height: 0 }).map_err(|e| err1("err.sshResize", e))
}

#[tauri::command]
fn stop_ssh_session(app: tauri::AppHandle, profile_id: String) -> Result<(), String> {
    let mut session = app.state::<SshSessions>().0.lock().unwrap().remove(&profile_id).ok_or("err.sshNotRunning")?;
    session.killer.kill().map_err(|e| err1("err.sshStop", e))
}

/// Schließt das aktuelle Terminalfenster nativ. So ist die Aktion nicht von
/// Webview-ACLs abhängig und beendet die zugehörige PTY auch dann, wenn die
/// Oberfläche gerade keine Solid-Cleanup-Routine mehr ausführen kann.
#[tauri::command]
fn close_terminal_session(app: tauri::AppHandle, window: tauri::WebviewWindow, profile_id: String) -> Result<(), String> {
    let session = app.state::<SshSessions>().0.lock().unwrap().remove(&profile_id);
    if let Some(mut session) = session {
        let _ = session.killer.kill();
    }
    window.close().map_err(|error| err1("err.terminalClose", error))
}

/// Minimiert ausschließlich das aktuelle Terminalfenster. Anders als beim
/// Hauptfenster gibt es bewusst keinen Ausblend-Rückfall: Die X11-App soll
/// sichtbar bleiben und die Konsole lediglich als Dock-Symbol weiterlaufen.
#[tauri::command]
fn minimize_terminal_window(window: tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let target = window.clone();
        let _ = window.run_on_main_thread(move || {
            use objc2::rc::Retained;
            use objc2_app_kit::NSWindow;
            let Ok(handle) = target.ns_window() else {
                return;
            };
            let Some(ns) = (unsafe { Retained::retain(handle.cast::<NSWindow>()) }) else {
                return;
            };
            ns.miniaturize(None);
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.minimize();
    }
}

impl Sessions {
    /// Meldet eine Sitzung an. `None`, wenn für dieses Profil bereits eine
    /// läuft – dann enthält der Rückgabewert deren Prozesskennung.
    ///
    /// Prüfen und Eintragen geschehen unter **einem** Lock. Zwei gleichzeitig
    /// eintreffende Deep-Links würden sonst beide "frei" sehen.
    fn claim(&self, id: &str) -> Result<(), Option<u32>> {
        let mut running = self.0.lock().map_err(|_| None)?;
        if let Some(&pid) = running.get(id) {
            return Err(Some(pid));
        }
        // 0 heisst "startet gerade"; die echte Kennung folgt mit `attach`.
        running.insert(id.to_string(), 0);
        Ok(())
    }

    fn attach(&self, id: &str, pid: u32) {
        if let Ok(mut running) = self.0.lock() {
            running.insert(id.to_string(), pid);
        }
    }

    fn release(&self, id: &str) {
        if let Ok(mut running) = self.0.lock() {
            running.remove(id);
        }
    }
}

/// Holt das Fenster einer bereits laufenden Sitzung nach vorne.
///
/// Kein Fehlerfall: Der Anwender wollte zu dieser Sitzung, und genau dorthin
/// bringen wir ihn. Eine Meldung wäre hier nur im Weg.
#[cfg(target_os = "macos")]
fn focus_running_session(profile_id: &str) {
    // Das Startbundle je Profil trägt diese Kennung (siehe `session_launcher`).
    let bundle = format!("com.remotedesk.session.{}", safe_identifier(profile_id));
    let _ = Command::new("/usr/bin/open")
        .args(["-b", &bundle])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(not(target_os = "macos"))]
fn focus_running_session(_profile_id: &str) {}

/// Eine per Deep-Link angeforderte Verbindung, die das Fenster noch nicht
/// abgeholt hat. Nötig für den Kaltstart: die URL trifft ein, bevor die
/// Oberfläche steht, sonst ginge sie verloren.
#[derive(Default)]
struct PendingLink(Mutex<Option<String>>);

/// Holt eine wartende Verbindung ab und leert den Speicher, damit dieselbe
/// Anforderung nicht zweimal ausgeführt wird.
#[tauri::command]
fn take_pending_link(state: tauri::State<'_, PendingLink>) -> Option<String> {
    state.0.lock().ok().and_then(|mut slot| slot.take())
}

/// Nimmt das eigene Fenster vom Schirm.
///
/// Wird nur nach einer aus DualBeam angeforderten Sitzung gerufen. Dort ist das
/// Fenster reines Mittel zum Zweck: es holt die Kennung ab und startet FreeRDP,
/// danach steht es nur noch im Weg. Geschlossen wird es bewusst *nicht* - dann
/// endete das Programm, und mit ihm der Mitschnitt von FreeRDPs Fehlerausgabe.
///
/// Ein Fehlschlag bleibt folgenlos: das ist Bequemlichkeit, keine Voraussetzung
/// fuer die Sitzung. Eine Meldung dafuer waere nur laestig.
#[tauri::command]
fn minimize_window(window: tauri::Window) {
    // Tauris `minimize()` meldet auf macOS Ok, klappt das Fenster aber nicht
    // zuverlaessig ein - gemessen blieb `is_minimized()` false, das Fenster
    // stand weiter auf dem Schirm. Auch AppKits `miniaturize` greift nur
    // manchmal (zwei Laeufe, zwei Ergebnisse), vermutlich weil das Fenster
    // gerade erst ueber LaunchServices aktiviert wurde. Deshalb der Rueckfall
    // aufs Ausblenden des ganzen Programms - das entspricht Cmd-H, wirkt immer
    // und braucht keine Bedienungshilfen-Rechte, weil sich das Programm selbst
    // ausblendet. Das Dock-Symbol bleibt, ein Klick holt das Fenster zurueck.
    #[cfg(target_os = "macos")]
    {
        let target = window.clone();
        let _ = window.run_on_main_thread(move || {
            use objc2::rc::Retained;
            use objc2_app_kit::{NSApplication, NSWindow};
            let Ok(handle) = target.ns_window() else { return };
            if handle.is_null() {
                return;
            }
            // Sicherheit: `ns_window()` liefert genau das NSWindow dieses
            // Fensters; wir leihen es uns nur fuer den einen Aufruf.
            let Some(ns) = (unsafe { Retained::retain(handle.cast::<NSWindow>()) }) else {
                return;
            };
            ns.miniaturize(None);
            if !ns.isMiniaturized() {
                if let Some(mtm) = objc2::MainThreadMarker::new() {
                    NSApplication::sharedApplication(mtm).hide(None);
                }
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.minimize();
    }
}

/// Holt die Profilverwaltung nach einem Dock-Klick zuverlässig zurück.
///
/// `minimize_window` fällt auf macOS bei einem zu frühen Miniaturisieren auf
/// `NSApplication::hide` zurück. Das blendet nicht nur das Hauptfenster,
/// sondern die gesamte App aus. Ein Dock-Klick löst in diesem Fall das
/// `Reopen`-Ereignis aus; wir müssen die App explizit einblenden und das
/// Fenster wiederherstellen, statt uns auf die Webview-Standardbehandlung zu
/// verlassen.
#[cfg(target_os = "macos")]
fn restore_main_window(app: &tauri::AppHandle) {
    use tauri::Manager;
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let target = window.clone();
    let _ = app.run_on_main_thread(move || {
        use objc2_app_kit::NSApplication;
        if let Some(mtm) = objc2::MainThreadMarker::new() {
            NSApplication::sharedApplication(mtm).unhideWithoutActivation();
        }
        let _ = target.unminimize();
        let _ = target.show();
        let _ = target.set_focus();
    });
}

/// Legt die angeforderte Verbindung ab und weckt das Fenster.
///
/// Die ID reist bewusst **nicht** im Ereignis mit. Sonst gäbe es zwei Quellen
/// für dieselbe Anforderung, und je nachdem, ob das Fenster gerade lauscht,
/// liefe sie doppelt oder gar nicht. So ist das Ereignis nur ein Weckruf; die
/// Oberfläche holt die ID stets über `take_pending_link` ab, das den Speicher
/// atomar leert.
fn remember_link(handle: &tauri::AppHandle, url: &tauri::Url) {
    use tauri::{Emitter, Manager};
    let Some(id) = profile_id_from_url(url) else { return };
    if let Ok(mut slot) = handle.state::<PendingLink>().0.lock() {
        *slot = Some(id);
    }
    let _ = handle.emit("deep-link-connect", ());
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.set_focus();
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(PendingLink::default())
        .manage(Sessions::default())
        .manage(SshSessions(Mutex::new(HashMap::new())))
        .manage(ProfileStore::default())
        .setup(|app| {
            use tauri_plugin_deep_link::DeepLinkExt;
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    remember_link(&handle, &url);
                }
            });
            // Anderer Weg zum selben Ziel, falls das Betriebssystem den Link
            // nicht als Ereignis nachreicht, sondern nur beim Start bereithält.
            // Auf macOS gemessen: liefert hier stets None, `on_open_url` feuert
            // auch beim Kaltstart zuverlässig. Für Windows und Linux, wo die URL
            // als Startargument ankommt, bleibt die Abfrage nötig.
            if let Ok(Some(urls)) = app.deep_link().get_current() {
                let handle = app.handle().clone();
                for url in urls {
                    remember_link(&handle, &url);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_profiles, save_profile, delete_profile, save_password, load_password, forget_password, save_gateway_password, load_gateway_password, forget_gateway_password, connect_profile, start_vnc_session, start_ssh_session, write_ssh_session, resize_ssh_session, stop_ssh_session, close_terminal_session, minimize_terminal_window, take_pending_link, minimize_window])
        .build(tauri::generate_context!())
        .expect("Fehler beim Start von RemoteDeskRDP");
    app.run(|app, event| {
            #[cfg(target_os = "macos")]
            if matches!(event, tauri::RunEvent::Reopen { .. }) {
                restore_main_window(app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deep-Links sind eine Aussenschnittstelle: Was hier durchrutscht, loest
    /// ungewollt eine Verbindung aus. Deshalb eng gepruefte Faelle.
    #[test]
    fn deep_link_urls_are_parsed_strictly() {
        let parse = |raw: &str| {
            tauri::Url::parse(raw).ok().as_ref().and_then(profile_id_from_url)
        };
        // Gueltig
        assert_eq!(parse("remotedesk://connect?id=abc-123"), Some("abc-123".into()));
        // Weitere Parameter stoeren nicht
        assert_eq!(parse("remotedesk://connect?x=1&id=abc"), Some("abc".into()));
        // Prozentkodierung wird aufgeloest
        assert_eq!(parse("remotedesk://connect?id=a%20b"), Some("a b".into()));
        // Fremdes Schema
        assert_eq!(parse("dualbeam://connect?id=abc"), None);
        // Unbekannte Aktion
        assert_eq!(parse("remotedesk://start?id=abc"), None);
        // Ohne Aktion
        assert_eq!(parse("remotedesk://?id=abc"), None);
        // Ohne oder mit leerer ID
        assert_eq!(parse("remotedesk://connect"), None);
        assert_eq!(parse("remotedesk://connect?id="), None);
        assert_eq!(parse("remotedesk://connect?id=%20%20"), None);
    }

    /// Fehler muessen als Code herausgehen, damit die Oberflaeche sie in der
    /// gewaehlten Sprache anzeigen kann. Ein deutscher Satz waere still
    /// unuebersetzbar -- diese Pruefung faellt vorher auf.
    #[test]
    fn errors_are_codes_not_prose() {
        let mut kaputt = profile();
        kaputt.name = String::new();
        let faelle: Vec<String> = vec![
            validate_profile(&kaputt).unwrap_err(),
            {
                let mut p = profile();
                p.host = String::new();
                validate_profile(&p).unwrap_err()
            },
            {
                let mut p = profile();
                p.width = 641;
                validate_profile(&p).unwrap_err()
            },
            {
                let mut p = profile();
                p.gateway_enabled = true;
                p.gateway_host = "gw:443".into();
                validate_profile(&p).unwrap_err()
            },
            {
                let mut p = profile();
                p.shared_folders = vec![SharedFolder { name: "a,b".into(), path: "/tmp".into() }];
                drive_arguments(&p).unwrap_err()
            },
            keychain_account("").unwrap_err(),
        ];
        for fall in faelle {
            let code = fall.split('\u{1f}').next().unwrap();
            assert!(
                code.starts_with("err.") && !code.contains(' '),
                "kein Fehlercode, sondern Klartext: {fall:?}"
            );
        }
    }

    /// Der Fenstertitel reist zeilenweise über stdin zu FreeRDP. Ein
    /// Zeilenumbruch im Profilnamen würde die Argumentliste zerreissen.
    #[test]
    fn window_title_falls_back_and_survives_control_characters() {
        let mut p = profile();
        p.name = "Linux".into();
        assert_eq!(window_title(&p), "Linux");
        // Leerer Name -> Host, damit der Titel nie leer bleibt
        p.name = "   ".into();
        assert_eq!(window_title(&p), "192.168.8.103");
        // Zeilenumbruch und Tabulator werden zu Leerzeichen
        p.name = " Büro\nWindows\t11 ".into();
        assert_eq!(window_title(&p), "Büro Windows 11");
        // und landet als genau ein Argument in der Liste
        p.name = "Linux\nboese".into();
        let args = rdp_arguments(&p, Backend::Sdl).unwrap();
        assert!(args.contains(&"/t:Linux boese".to_string()), "{args:?}");
        assert!(args.iter().all(|a| !a.contains('\n')));
    }

    /// Der Profilname wird zum Ordnernamen und landet in XML. Beides sind
    /// Aussenschnittstellen: der Name kommt aus einer Datei, die auch von Hand
    /// entstehen kann.
    #[cfg(target_os = "macos")]
    #[test]
    fn launcher_names_are_sanitised() {
        // Pfadtrenner und Doppelpunkt duerfen keinen Unterordner erzeugen
        assert_eq!(safe_component("Buero/Linux"), "Buero_Linux");
        assert_eq!(safe_component("A:B"), "A_B");
        // Ein fuehrender Punkt versteckt den Ordner
        assert_eq!(safe_component(".versteckt"), "versteckt");
        // Leerer Name faellt auf einen Ersatz zurueck
        assert_eq!(safe_component("   "), "Sitzung");
        // Kennungen duerfen nur Buchstaben, Ziffern, Punkt und Bindestrich
        assert_eq!(safe_identifier("2ae2238f-01e2-4918"), "2ae2238f-01e2-4918");
        assert_eq!(safe_identifier("a b/c"), "a-b-c");
        // XML: drei Zeichen wuerden das Plist zerreissen
        assert_eq!(xml_escape("Fritz & <Co>"), "Fritz &amp; &lt;Co&gt;");
    }

    fn profile() -> RemoteProfile {
        RemoteProfile {
            id: "id".into(),
            name: "Test".into(),
            protocol: Protocol::Rdp,
            host: "192.168.8.103".into(),
            username: String::new(),
            domain: String::new(),
            rdp_tcp_port: 3389,
            rdp_udp_port: 3389,
            vnc_port: DEFAULT_VNC_PORT,
            ssh_port: DEFAULT_SSH_PORT,
            ssh_terminal: DEFAULT_SSH_TERMINAL.into(),
            x11_forwarding: false,
            x11_command: String::new(),
            x11_close_xquartz: false,
            object_endpoint: String::new(),
            object_region: String::new(),
            object_access_key: String::new(),
            object_container: String::new(),
            object_path_style: true,
            swift_project: String::new(),
            swift_user_domain: String::new(),
            swift_project_domain: String::new(),
            swift_identity_path: DEFAULT_SWIFT_IDENTITY_PATH.into(),
            swift_auth_version: SwiftAuthVersion::V3,
            udp_preferred: true,
            clipboard: true,
            audio: false,
            display_mode: DisplayMode::Window,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            resize_behavior: ResizeBehavior::Dynamic,
            color_depth: ColorDepth::Auto,
            shared_folders: Vec::new(),
            printer: false,
            smartcard: false,
            video: false,
            auto_reconnect: true,
            gateway_enabled: false,
            gateway_host: String::new(),
            gateway_port: DEFAULT_GATEWAY_PORT,
            gateway_username: String::new(),
            gateway_domain: String::new(),
            certificate_mode: CertificateMode::Tofu,
            updated_at: "2026-07-30T00:00:00Z".into(),
        }
    }

    #[test]
    fn a_shared_folder_becomes_a_drive_redirect() {
        let mut profile = profile();
        profile.shared_folders = vec![SharedFolder {
            name: "Austausch".into(),
            path: "/Users/nojan/Austausch".into(),
        }];
        let arguments = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        assert!(arguments.contains(&"/drive:Austausch,/Users/nojan/Austausch".to_string()));
    }

    /// Ohne Schalter darf kein Druckerargument entstehen, mit Schalter genau
    /// eines -- FreeRDP reicht dann alle über CUPS bekannten Drucker weiter.
    #[test]
    fn the_printer_switch_controls_the_redirect() {
        let mut profile = profile();
        assert!(!rdp_arguments(&profile, Backend::Sdl)
            .expect("Argumente")
            .contains(&"/printer".to_string()));
        profile.printer = true;
        assert!(rdp_arguments(&profile, Backend::Sdl)
            .expect("Argumente")
            .contains(&"/printer".to_string()));
    }

    /// Jeder Geräteschalter darf nur sein eigenes Argument erzeugen; ein
    /// versehentlich immer gesetzter Kanal fiele sonst erst am Ziel auf.
    #[test]
    fn device_switches_stay_independent() {
        let mut profile = profile();
        let off = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        for flag in ["/printer", "/smartcard", "/video"] {
            assert!(!off.contains(&flag.to_string()), "{flag} ohne Schalter gesetzt");
        }
        profile.smartcard = true;
        let only_smartcard = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        assert!(only_smartcard.contains(&"/smartcard".to_string()));
        assert!(!only_smartcard.contains(&"/video".to_string()));
        assert!(!only_smartcard.contains(&"/printer".to_string()));
        profile.video = true;
        assert!(rdp_arguments(&profile, Backend::Sdl)
            .expect("Argumente")
            .contains(&"/video".to_string()));
    }

    /// Bildet FreeRDPs `unescape()` (client/common/cmdline.c) nach: ein
    /// Backslash maskiert das folgende Zeichen und fällt selbst weg.
    fn freerdp_unescape(value: &str) -> String {
        let mut out = String::new();
        let mut escaped = false;
        for character in value.chars() {
            if character == '\\' && !escaped {
                escaped = true;
                continue;
            }
            out.push(character);
            escaped = false;
        }
        out
    }

    /// Der entscheidende Nachweis: Was maskiert wird, muss FreeRDPs
    /// Gegenstück wieder unverändert herstellen -- sonst ginge ein falsches
    /// Kennwort über die Leitung, ohne dass jemand etwas merkt.
    #[test]
    fn escaping_survives_freerdps_own_unescape() {
        for original in [
            "geheim",
            "ge,heim",
            r"ge\heim",
            "ge\"heim",
            "ge'heim",
            r#"a,b\c"d'e"#,
            r"\\",
            ",,,",
        ] {
            let escaped = escape_gateway_value(original);
            assert_eq!(
                freerdp_unescape(&escaped),
                original,
                "Rundlauf verändert {original:?} (maskiert: {escaped:?})"
            );
        }
    }




    #[test]
    fn a_gateway_is_only_added_when_switched_on() {
        let mut profile = profile();
        profile.gateway_host = "gw.example.com".into();
        assert!(
            rdp_arguments(&profile, Backend::Sdl)
                .expect("Argumente")
                .iter()
                .all(|argument| !argument.starts_with("/gateway:")),
            "Gateway ohne Schalter gesetzt"
        );
        profile.gateway_enabled = true;
        let arguments = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        assert!(arguments.contains(&"/gateway:g:gw.example.com:443".to_string()));
    }

    /// Ohne eigene Anmeldedaten meldet FreeRDP das Gateway mit denen der
    /// Sitzung an (`GatewayUseSameCredentials`). Es darf deshalb kein leeres
    /// `u:`/`d:` mitgehen -- das würde genau diesen Rückfall abschalten.
    #[test]
    fn empty_gateway_credentials_are_left_out() {
        let mut profile = profile();
        profile.gateway_enabled = true;
        profile.gateway_host = "gw.example.com".into();
        profile.gateway_username = "   ".into();
        profile.gateway_domain = String::new();
        let arguments = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        let gateway = arguments
            .iter()
            .find(|argument| argument.starts_with("/gateway:"))
            .expect("Gateway-Argument");
        assert_eq!(gateway, "/gateway:g:gw.example.com:443");
    }

    #[test]
    fn gateway_credentials_and_port_are_passed_on() {
        let mut profile = profile();
        profile.gateway_enabled = true;
        profile.gateway_host = "gw.example.com".into();
        profile.gateway_port = 8443;
        profile.gateway_username = "max".into();
        profile.gateway_domain = "CONTOSO".into();
        let arguments = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        assert!(arguments.contains(&"/gateway:g:gw.example.com:8443,u:max,d:CONTOSO".to_string()));
    }

    /// Ein Doppelpunkt im Hostfeld ergäbe zwei Portangaben, Leerzeichen
    /// zerrissen das Argument. Beides muss auffallen, nicht durchrutschen.
    #[test]
    fn a_malformed_gateway_host_is_refused() {
        let mut profile = profile();
        profile.gateway_enabled = true;
        for host in ["gw.example.com:443", "gw example.com", "   "] {
            profile.gateway_host = host.into();
            assert!(
                gateway_arguments(&profile).is_err() || validate_profile(&profile).is_err(),
                "Hostname {host:?} wurde angenommen"
            );
        }
    }

    #[test]
    fn profiles_without_gateway_fields_get_defaults() {
        let stored = r#"{
            "id": "alt", "name": "Alt", "host": "10.0.0.1", "username": "", "domain": "",
            "rdpTcpPort": 3389, "rdpUdpPort": 3389, "udpPreferred": true,
            "clipboard": true, "audio": false,
            "certificateMode": "tofu", "updatedAt": "2026-01-01T00:00:00Z"
        }"#;
        let profile: RemoteProfile = serde_json::from_str(stored).expect("Altprofil lesbar");
        assert!(!profile.gateway_enabled);
        assert_eq!(profile.gateway_port, DEFAULT_GATEWAY_PORT);
        assert!(gateway_arguments(&profile).expect("Argumente").is_empty());
    }

    #[test]
    fn several_folders_each_get_their_own_redirect() {
        let mut profile = profile();
        profile.shared_folders = vec![
            SharedFolder { name: "A".into(), path: "/tmp/a".into() },
            SharedFolder { name: "B".into(), path: "/tmp/b".into() },
        ];
        let arguments = drive_arguments(&profile).expect("Argumente");
        assert_eq!(arguments, vec!["/drive:A,/tmp/a", "/drive:B,/tmp/b"]);
    }

    /// Ein Komma verschöbe die Trennung zwischen Name und Pfad.
    #[test]
    fn a_comma_in_a_share_is_refused() {
        let mut profile = profile();
        profile.shared_folders = vec![SharedFolder {
            name: "Bilder, privat".into(),
            path: "/tmp/bilder".into(),
        }];
        assert!(drive_arguments(&profile).is_err());
    }

    #[test]
    fn an_incomplete_share_is_refused() {
        let mut profile = profile();
        profile.shared_folders = vec![SharedFolder { name: "  ".into(), path: "/tmp/a".into() }];
        assert!(drive_arguments(&profile).is_err());
    }

    #[test]
    fn line_breaks_cannot_create_extra_freerdp_arguments() {
        let mut profile = profile();
        profile.username = "alice\n/cert:ignore".into();
        assert_eq!(validate_profile(&profile), Err("err.argumentLineBreak".into()));

        profile.username.clear();
        profile.shared_folders = vec![SharedFolder {
            name: "Austausch\n/drive:unwanted,/tmp".into(),
            path: "/tmp/a".into(),
        }];
        assert_eq!(drive_arguments(&profile), Err("err.argumentLineBreak".into()));
    }

    #[test]
    fn profiles_without_shares_stay_without_drive_options() {
        let arguments = rdp_arguments(&profile(), Backend::Sdl).expect("Argumente");
        assert!(!arguments.iter().any(|a| a.starts_with("/drive")));
    }

    #[test]
    fn the_default_leaves_the_colour_depth_to_the_server() {
        let arguments = display_arguments(&profile(), Backend::Sdl);
        assert!(!arguments.iter().any(|a| a.starts_with("/bpp")));
    }

    #[test]
    fn a_chosen_colour_depth_is_passed_on() {
        let mut profile = profile();
        profile.color_depth = ColorDepth::Bits16;
        let arguments = display_arguments(&profile, Backend::Sdl);
        assert!(arguments.contains(&"/bpp:16".to_string()));
    }

    /// FreeRDP lehnt jeden anderen Wert mit einem Argumentfehler ab.
    #[test]
    fn every_colour_depth_is_one_freerdp_accepts() {
        for depth in [
            ColorDepth::Bits32,
            ColorDepth::Bits24,
            ColorDepth::Bits16,
            ColorDepth::Bits15,
            ColorDepth::Bits8,
        ] {
            assert!(matches!(depth.bits(), Some(32 | 24 | 16 | 15 | 8)));
        }
    }

    #[test]
    fn profiles_without_a_colour_depth_default_to_automatic() {
        let json = r#"{"id":"a","name":"n","protocol":"rdp","host":"h","username":"","domain":"",
            "rdpTcpPort":3389,"rdpUdpPort":3389,"vncPort":5900,"udpPreferred":true,
            "clipboard":true,"audio":false,"certificateMode":"tofu","updatedAt":"x"}"#;
        let profile: RemoteProfile = serde_json::from_str(json).expect("Profil lesbar");
        assert_eq!(profile.color_depth, ColorDepth::Auto);
    }

    #[test]
    fn window_mode_sets_the_requested_resolution() {
        let arguments = display_arguments(&profile(), Backend::Sdl);
        assert_eq!(arguments, vec!["/size:1600x1000", "+dynamic-resolution"]);
    }

    #[test]
    fn scaling_and_dynamic_resolution_are_never_combined() {
        let mut scaled = profile();
        scaled.resize_behavior = ResizeBehavior::Scale;
        let arguments = display_arguments(&scaled, Backend::Sdl);
        assert!(arguments.contains(&"/smart-sizing".to_string()));
        assert!(!arguments.iter().any(|item| item.contains("dynamic-resolution")));
    }

    #[test]
    fn fullscreen_replaces_the_fixed_size() {
        let mut fullscreen = profile();
        fullscreen.display_mode = DisplayMode::Fullscreen;
        let arguments = display_arguments(&fullscreen, Backend::Sdl);
        assert!(arguments.contains(&"+f".to_string()));
        assert!(!arguments.iter().any(|item| item.starts_with("/size:")));
    }

    #[test]
    fn work_area_uses_the_usable_screen() {
        let mut workarea = profile();
        workarea.display_mode = DisplayMode::WorkArea;
        assert!(display_arguments(&workarea, Backend::Sdl).contains(&"+workarea".to_string()));
    }

    #[test]
    fn odd_window_sizes_are_rejected() {
        let mut odd = profile();
        odd.width = 1601;
        assert!(validate_profile(&odd).is_err());
    }

    #[test]
    fn the_cocoa_client_scales_instead_of_resizing_the_session() {
        // Der native Client kennt keinen Display-Control-Kanal, deshalb muss
        // aus der dynamischen Auflösung eine Skalierung werden.
        let arguments = display_arguments(&profile(), Backend::Cocoa);
        assert!(arguments.contains(&"/smart-sizing".to_string()));
        assert!(!arguments.iter().any(|item| item.contains("dynamic-resolution")));
    }

    #[test]
    fn a_fixed_session_never_gets_a_resize_option() {
        let mut fixed = profile();
        fixed.resize_behavior = ResizeBehavior::Fixed;
        for backend in [Backend::Cocoa, Backend::Sdl, Backend::X11] {
            assert_eq!(display_arguments(&fixed, backend), vec!["/size:1600x1000"], "{backend:?}");
        }
    }

    #[test]
    fn ssh_terminal_type_is_profiled_and_validated() {
        let mut ssh = profile();
        ssh.protocol = Protocol::Ssh;
        ssh.ssh_terminal = "screen-256color".into();
        assert!(validate_profile(&ssh).is_ok());

        ssh.ssh_terminal = "xterm 256color".into();
        assert_eq!(validate_profile(&ssh).unwrap_err(), "err.sshTerminalInvalid");

        ssh.ssh_terminal.clear();
        assert_eq!(validate_profile(&ssh).unwrap_err(), "err.sshTerminalInvalid");
    }

    #[test]
    fn x11_command_is_an_optional_single_line_profile_value() {
        let mut ssh = profile();
        ssh.protocol = Protocol::Ssh;
        ssh.x11_forwarding = true;
        ssh.x11_command = "gedit --new-window".into();
        assert!(validate_profile(&ssh).is_ok());

        ssh.x11_command = "gedit\nshutdown -h now".into();
        assert_eq!(validate_profile(&ssh).unwrap_err(), "err.argumentLineBreak");

        ssh.x11_command = "x".repeat(1025);
        assert_eq!(validate_profile(&ssh).unwrap_err(), "err.x11CommandInvalid");
    }

    #[test]
    fn object_storage_profiles_need_an_https_or_http_endpoint() {
        let mut s3 = profile();
        s3.protocol = Protocol::S3;
        s3.host.clear();
        s3.object_endpoint = "https://s3.example.test".into();
        s3.object_region = "us-east-1".into();
        s3.object_access_key = "access".into();
        assert!(validate_profile(&s3).is_ok());
        s3.object_endpoint = "s3.example.test".into();
        assert_eq!(validate_profile(&s3).unwrap_err(), "err.objectEndpointInvalid");
    }

    #[test]
    fn s3_path_style_keeps_object_prefixes_encoded() {
        let mut s3 = profile();
        s3.protocol = Protocol::S3;
        s3.object_endpoint = "https://s3.example.test/api".into();
        let url = s3_url(&s3, "archive", "reports/July 2026.pdf", Some("prefix=reports%2F")).unwrap();
        assert_eq!(url.as_str(), "https://s3.example.test/api/archive/reports/July%202026.pdf?prefix=reports%2F");
    }

    #[test]
    fn swift_location_and_keystone_path_form_the_auth_url() {
        assert_eq!(
            swift_auth_url("https://swiss-backup02.infomaniak.com/", "/identity/v3", SwiftAuthVersion::V3).unwrap().as_str(),
            "https://swiss-backup02.infomaniak.com/identity/v3/auth/tokens"
        );
        assert_eq!(
            swift_auth_url("https://swift.example.test/identity/v3", "/identity/v3", SwiftAuthVersion::V3).unwrap().as_str(),
            "https://swift.example.test/identity/v3/auth/tokens"
        );
        assert_eq!(
            swift_auth_url("https://swift.example.test/", "/v2.0", SwiftAuthVersion::V2).unwrap().as_str(),
            "https://swift.example.test/v2.0/tokens"
        );
    }

    #[test]
    fn sftp_and_mosh_reuse_the_safe_ssh_profile_fields() {
        let mut sftp = profile();
        sftp.protocol = Protocol::Sftp;
        sftp.rdp_tcp_port = 0;
        sftp.rdp_udp_port = 0;
        assert!(validate_profile(&sftp).is_ok());

        let mut mosh = profile();
        mosh.protocol = Protocol::Mosh;
        mosh.ssh_port = 0;
        assert_eq!(validate_profile(&mosh).unwrap_err(), "err.portRange");
    }

    #[test]
    fn backends_are_recognised_by_file_name() {
        assert_eq!(backend_kind(Path::new("/a/MacFreeRDP")), Backend::Cocoa);
        assert_eq!(backend_kind(Path::new("/a/xfreerdp")), Backend::X11);
        assert_eq!(backend_kind(Path::new("/a/sdl-freerdp")), Backend::Sdl);
    }

    #[test]
    fn profiles_without_display_fields_get_defaults() {
        // Enthaelt absichtlich die abgelegten Felder "protocol" und "vncPort":
        // Profile aus der Zeit mit VNC-Unterstuetzung liegen so auf der Platte
        // und muessen weiterhin gelesen werden koennen.
        let legacy = r#"{
            "id": "id", "name": "Alt", "protocol": "rdp", "host": "10.0.0.1",
            "username": "", "domain": "", "rdpTcpPort": 3389, "rdpUdpPort": 3389,
            "vncPort": 5900, "udpPreferred": true, "clipboard": true, "audio": true,
            "dynamicResolution": true, "certificateMode": "tofu",
            "updatedAt": "2026-07-30T00:00:00Z"
        }"#;
        let parsed: RemoteProfile = serde_json::from_str(legacy).expect("Altprofil muss lesbar bleiben");
        assert_eq!(parsed.display_mode, DisplayMode::Window);
        assert_eq!(parsed.width, DEFAULT_WIDTH);
        assert_eq!(parsed.resize_behavior, ResizeBehavior::Dynamic);
        // Anders als die uebrigen neuen Schalter ist dieser fuer Bestandsprofile
        // **an**: er wurde eingefuehrt, weil genau diese Profile abbrachen.
        assert!(parsed.auto_reconnect);
    }

    #[test]
    fn the_log_level_is_pinned_not_inherited() {
        // "Network disconnect!" und "Attempting reconnect" sind INFO und damit
        // der einzige Beleg, ob eine Wiederverbindung stattfand. Gemessen ist
        // INFO bereits FreeRDPs Vorgabe; festgeschrieben wird sie trotzdem,
        // damit eine spaetere Fassung sie nicht stillschweigend absenkt.
        let arguments = rdp_arguments(&profile(), Backend::Sdl).expect("Argumente");
        assert!(arguments.iter().any(|a| a == "/log-level:INFO"));
    }

    #[test]
    fn auto_reconnect_is_a_profile_switch() {        let mut profile = profile();
        profile.auto_reconnect = true;
        let on = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        assert!(on.iter().any(|a| a == "+auto-reconnect"));
        profile.auto_reconnect = false;
        let off = rdp_arguments(&profile, Backend::Sdl).expect("Argumente");
        assert!(!off.iter().any(|a| a.contains("auto-reconnect")));
    }

    #[test]
    fn log_generations_shift_instead_of_being_overwritten() {
        let dir = std::env::temp_dir().join(format!("rdlogtest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("Testordner");
        let path = dir.join("profil.log");

        // Drei Sitzungen nacheinander; die aelteste muss nach hinten wandern.
        for run in ["erste", "zweite", "dritte"] {
            rotate_logs(&path);
            fs::write(&path, run).expect("schreiben");
        }
        assert_eq!(fs::read_to_string(&path).unwrap(), "dritte");
        assert_eq!(fs::read_to_string(dir.join("profil.log.1")).unwrap(), "zweite");
        assert_eq!(fs::read_to_string(dir.join("profil.log.2")).unwrap(), "erste");

        // Ueber die Grenze hinaus darf nichts liegen bleiben.
        for _ in 0..LOG_GENERATIONS + 2 {
            rotate_logs(&path);
            fs::write(&path, "x").expect("schreiben");
        }
        let mut name = path.as_os_str().to_os_string();
        name.push(format!(".{}", LOG_GENERATIONS + 1));
        assert!(!PathBuf::from(name).exists(), "Es duerfen hoechstens {LOG_GENERATIONS} Generationen liegen");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_timestamp_is_readable() {
        let stamp = local_timestamp();
        // Form "2026-08-02 18:01:26 +0200" -- die Jahreszahl genuegt als Probe,
        // der Rest haengt an der Systemzeitzone.
        assert!(stamp.len() >= 19, "unerwartet kurz: {stamp}");
        assert!(stamp.starts_with("20"), "kein Datum: {stamp}");
    }

    #[test]
    fn a_legacy_vnc_profile_is_reactivated() {
        // Die frühere App schrieb bereits VNC-Profile. Version 0.2.0 muss sie
        // nicht migrieren oder verlieren, sondern direkt wieder öffnen können.
        let legacy = r#"{
            "id": "id", "name": "Alter Mac", "protocol": "vnc", "host": "10.0.0.9",
            "username": "", "domain": "", "rdpTcpPort": 3389, "rdpUdpPort": 3389,
            "vncPort": 5900, "udpPreferred": true, "clipboard": true, "audio": false,
            "certificateMode": "tofu", "updatedAt": "2026-07-30T00:00:00Z"
        }"#;
        let parsed: RemoteProfile = serde_json::from_str(legacy).expect("VNC-Profil muss lesbar bleiben");
        assert_eq!(parsed.name, "Alter Mac");
        assert_eq!(parsed.host, "10.0.0.9");
        assert_eq!(parsed.protocol, Protocol::Vnc);
        assert_eq!(parsed.vnc_port, DEFAULT_VNC_PORT);
    }

    #[test]
    fn profiles_without_a_protocol_remain_rdp_profiles() {
        let stored = r#"{
            "id": "alt", "name": "Alt", "host": "10.0.0.1", "username": "", "domain": "",
            "rdpTcpPort": 3389, "rdpUdpPort": 3389, "udpPreferred": true,
            "clipboard": true, "audio": false,
            "certificateMode": "tofu", "updatedAt": "2026-01-01T00:00:00Z"
        }"#;
        let parsed: RemoteProfile = serde_json::from_str(stored).expect("Altprofil lesbar");
        assert_eq!(parsed.protocol, Protocol::Rdp);
        assert_eq!(parsed.vnc_port, DEFAULT_VNC_PORT);
    }

    #[test]
    fn vnc_targets_preserve_host_and_port() {
        let mut vnc = profile();
        vnc.protocol = Protocol::Vnc;
        vnc.host = "vnc.example.net".into();
        vnc.vnc_port = 5901;
        assert_eq!(vnc_target(&vnc), "vnc.example.net:5901");
        vnc.host = "2001:db8::1".into();
        assert_eq!(vnc_target(&vnc), "[2001:db8::1]:5901");
    }

    #[test]
    fn vnc_requires_a_port_but_not_rdp_settings() {
        let mut vnc = profile();
        vnc.protocol = Protocol::Vnc;
        vnc.rdp_tcp_port = 0;
        vnc.rdp_udp_port = 0;
        assert!(validate_profile(&vnc).is_ok());
        vnc.vnc_port = 0;
        assert_eq!(validate_profile(&vnc), Err("err.portRange".into()));
    }

    /// Der Kern der Abbruch-Behebung: Eine zweite Sitzung zum selben Profil
    /// darf es nicht geben. Der RDP-Server traennt sonst die aeltere ohne
    /// Meldung -- genau das gemeldete Fehlerbild "Sitzung bricht ab".
    #[test]
    fn a_profile_can_only_hold_one_session() {
        let sessions = Sessions::default();

        // Erste Sitzung: frei.
        assert!(sessions.claim("linux").is_ok());
        sessions.attach("linux", 4711);

        // Zweite Sitzung zum selben Profil: abgewiesen, mit der laufenden PID.
        assert_eq!(sessions.claim("linux"), Err(Some(4711)));

        // Ein anderes Profil ist davon unberuehrt -- Linux und Windows sind
        // verschiedene Server und stoeren sich nicht.
        assert!(sessions.claim("windows").is_ok());

        // Nach dem Ende der Sitzung ist das Profil wieder frei.
        sessions.release("linux");
        assert!(sessions.claim("linux").is_ok());
    }

    /// Waehrend des Starts ist die PID noch unbekannt (0). Auch dann muss ein
    /// zweiter Versuch abgewiesen werden -- sonst schluepfen zwei gleichzeitig
    /// eintreffende Deep-Links beide durch, und genau das erzeugte die vier
    /// Starts in 16 Sekunden aus dem Systemprotokoll.
    #[test]
    fn a_session_is_reserved_before_the_process_exists() {
        let sessions = Sessions::default();
        assert!(sessions.claim("linux").is_ok());
        assert_eq!(sessions.claim("linux"), Err(Some(0)));
    }

    /// Das Sitzungsprotokoll darf keine Pfadtrenner aus der Profil-ID erben.
    /// Die ID stammt aus einer Datei, die auch von Hand entstehen kann.
    #[test]
    fn the_session_log_stays_inside_its_folder() {
        let path = session_log_path("../../etc/passwd").expect("Pfad erwartet");
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(!name.contains('/'), "Dateiname enthaelt Pfadtrenner: {name}");
        assert!(name.ends_with(".log"));
        assert_eq!(path.parent().unwrap().file_name().unwrap(), "logs");
    }
}
