# Changelog

All notable changes to RemoteDeskRDP are documented here.

## 0.7.3 – 2026-10-09

- Update Tauri CLI and plugins: updater 2.13, dialog 2.8, deep-link 2.6,
  fs 2.6.

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
