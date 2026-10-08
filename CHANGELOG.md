# Changelog

All notable changes to RemoteDeskRDP are documented here.

## Unreleased

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
