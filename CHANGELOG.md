# Changelog

All notable changes to RemoteDeskRDP are documented here.

## Unreleased

- Fix: RDP connection failures that occur after the initial start-up check
  (e.g. TCP timeouts or a denied macOS "Local Network" permission) are now
  reported in the UI as "Connection failed: FreeRDP exited: …" instead of
  silently leaving the status at "Connected". Sessions closed by the user
  are not reported.

## 0.7.5 – 2026-10-09

- Fix: profiles with a saved monitor list but "Multiple monitors" switched off
  failed to connect after 0.7.4 (`err.monitorsNeedMultimon`). The list is now
  simply ignored without the switch and cleared when the switch is turned off.

## 0.7.4 – 2026-10-09

- Microsoft Entra ID sign-in (`/sec:aad`): the Microsoft login page opens in
  its own window and the authorization code is passed to FreeRDP. SDL backend
  only.
- Multiple monitors (`/multimon`, `/monitors`) with monitor detection in the
  profile editor. SDL backend only; not yet tested on real multi-monitor
  setups.
- Code review follow-up: stricter profile validation (host with port suffix,
  Entra ID user name, monitor list and scaling in multi-monitor mode), safer
  session bookkeeping and log files, launcher plist fixes, typed noVNC
  bindings, updater resources released on decline, and refreshed help texts
  for Entra ID, multiple monitors, keyboard layout and the updater.

## 0.7.3 – 2026-10-09

- Update Tauri CLI and plugins: updater 2.13, dialog 2.8, deep-link 2.6,
  fs 2.6.
- New RDP options: microphone redirection, device and desktop scaling for
  Retina displays, administrator/console session, keyboard layout and time
  zone. RemoteApp is deferred: the SDL client cannot render RAIL windows yet.
- The app and the FreeRDP clients now carry the microphone entitlement and
  usage description, so macOS asks for permission.

## 0.7.2 – 2026-10-09

- Fix the updater never installing anything: WKWebView ignores `alert()` and
  `confirm()`, so the update prompt silently answered "no". Update, deep-link
  and delete confirmations now use native dialogs. Installations of 0.7.0 and
  0.7.1 need a one-time manual update from the DMG.

## 0.7.1 – 2026-10-09

- Fix RDP connections aborting at launch (SIGABRT, "Library missing"): the
  bundled FreeRDP libraries no longer use symlinks, which the app bundle turned
  into copies with mismatching install names.

## 0.7.0 – 2026-10-08

- Add an in-app updater: RemoteDeskRDP checks for new releases at start and via
  **RemoteDeskRDP → Check for Updates…**, verifies the signed update and
  restarts into the new version.
- `scripts/notarize.sh` now produces the signed updater archive and
  `latest.json` for each release.

## 0.6.3 – 2026-08-29

- Publish RemoteDeskRDP as an independent open-source project.
- License the application's own source code under the MIT License.
- Replace the former end-user license agreement in the application with the
  MIT License text.
- Remove obsolete S3 and OpenStack Swift references from the application help
  and public project description.
- Add repository, contribution and security documentation.
- Bundle the patched FreeRDP backend in the signed and notarized macOS release.
