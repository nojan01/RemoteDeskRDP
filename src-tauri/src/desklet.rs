//! ICMP-Prüfung ohne Shell; ausschließlich validierte IP-Adressen sind erlaubt.
use std::net::IpAddr;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tauri::Manager;

static OPEN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Plugin-Ereignisse halten Tauris PluginStore-Sperre. Fenster dürfen erst
/// außerhalb dieses Callbacks entstehen, weil der Builder dieselbe Sperre braucht.
pub(crate) fn request_open(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = open_vm_desklet(app.clone()) {
            eprintln!("VM desklet could not open: {error}");
        }
        crate::menubar::refresh_dock(&app);
    });
}

pub(crate) fn is_desklet_url(url: &tauri::Url) -> bool {
    url.scheme() == "remotedesk"
        && url.host_str() == Some("desklet")
        && matches!(url.path(), "" | "/")
        && url.query().is_none()
        && url.fragment().is_none()
}

fn parse_address(address: &str) -> Result<IpAddr, String> {
    address.parse().map_err(|_| "desklet.invalidIp".into())
}

fn classify_result(code: Option<i32>, stderr: &[u8], timed_out: bool) -> Result<bool, String> {
    if code == Some(0) {
        Ok(true)
    } else if !stderr.is_empty() {
        Err("desklet.pingError".into())
    } else if timed_out || code == Some(2) {
        Ok(false)
    } else {
        Err("desklet.pingError".into())
    }
}

fn ping(address: IpAddr) -> Result<bool, String> {
    // macOS verwendet für IPv6 ein eigenes Programm. Das äußere Zeitlimit
    // begrenzt auch fehlgeschlagene Sendeversuche und räumt den Prozess auf.
    let executable = if address.is_ipv4() {
        "/sbin/ping"
    } else {
        "/sbin/ping6"
    };
    let mut child = Command::new(executable)
        .args(["-n", "-c", "1", &address.to_string()])
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "desklet.pingError".to_string())?;
    let started = Instant::now();
    let timed_out = loop {
        match child.try_wait() {
            Ok(Some(_)) => break false,
            Ok(None) if started.elapsed() < Duration::from_secs(2) => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                break true;
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("desklet.pingError".into());
            }
        }
    };
    let output = child
        .wait_with_output()
        .map_err(|_| "desklet.pingError".to_string())?;
    classify_result(output.status.code(), &output.stderr, timed_out)
}

#[tauri::command]
pub fn validate_vm_address(address: String) -> Result<String, String> {
    parse_address(&address).map(|address| address.to_string())
}

#[tauri::command]
pub async fn ping_vm(address: String) -> Result<bool, String> {
    let address = parse_address(&address)?;
    tauri::async_runtime::spawn_blocking(move || ping(address))
        .await
        .map_err(|_| "desklet.pingError".to_string())?
}

#[tauri::command(async)]
pub fn open_vm_desklet(app: tauri::AppHandle) -> Result<(), String> {
    // Autostart und Desktop-Link können gleichzeitig eintreffen. Nur ein
    // Aufruf darf ein neues Fenster mit dem festen Label erzeugen.
    let _guard = OPEN_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(window) = app.get_webview_window("vm-desklet") {
        window.show().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        "vm-desklet",
        tauri::WebviewUrl::App("index.html?desklet".into()),
    )
    .title("VM Ping")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .inner_size(350.0, 470.0)
    .resizable(false)
    .always_on_top(true)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desklet_links_are_separate_from_connection_links() {
        for link in ["remotedesk://desklet", "remotedesk://desklet/"] {
            assert!(is_desklet_url(&tauri::Url::parse(link).unwrap()));
        }
        for link in [
            "https://desklet",
            "remotedesk://connect?id=vm-desklet",
            "remotedesk://desklet/other",
            "remotedesk://desklet?id=abc",
        ] {
            assert!(!is_desklet_url(&tauri::Url::parse(link).unwrap()));
        }
    }

    #[test]
    fn only_ip_literals_can_be_passed_to_ping() {
        for address in ["127.0.0.1", "192.168.1.20", "::1", "2001:db8::1"] {
            assert!(parse_address(address).is_ok());
        }
        for address in [
            "",
            "localhost",
            "-f",
            "1.2.3.999",
            "127.0.0.1; touch /tmp/x",
            " 127.0.0.1",
        ] {
            assert!(parse_address(address).is_err());
        }
    }

    #[test]
    fn no_reply_and_operating_system_errors_are_distinct() {
        assert_eq!(classify_result(Some(0), b"", false), Ok(true));
        assert_eq!(classify_result(Some(2), b"", false), Ok(false));
        assert_eq!(classify_result(None, b"", true), Ok(false));
        assert!(classify_result(Some(2), b"ping: sendto: Operation not permitted", false).is_err());
        assert!(classify_result(None, b"ping: sendto: Operation not permitted", true).is_err());
        assert!(classify_result(Some(64), b"", false).is_err());
        assert!(classify_result(None, b"", false).is_err());
    }

    #[test]
    #[ignore = "Requires permission to send ICMP packets on the host"]
    fn loopback_answers_over_ipv4_and_ipv6() {
        assert_eq!(ping("127.0.0.1".parse().unwrap()), Ok(true));
        assert_eq!(ping("::1".parse().unwrap()), Ok(true));
    }
}
