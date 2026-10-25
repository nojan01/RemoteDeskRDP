# Signieren, Notarisieren und Update-Release

RemoteDeskRDP wird mit der Developer ID signiert. App und DMG werden getrennt
bei Apple notarisiert. Der vorhandene Updater-Schlüssel muss erhalten bleiben,
damit bereits installierte Versionen neue Updates prüfen können.

## Voraussetzungen

- Developer-ID-Identität im macOS-Schlüsselbund, passend zu
  `bundle.macOS.signingIdentity` in `src-tauri/tauri.conf.json`.
- Notarisierungsprofil `remotedesk-notary` im Schlüsselbund. Abweichender Name:
  `NOTARY_PROFILE`.
- Vorhandener privater Updater-Schlüssel unter
  `~/.tauri/remotedesk-updater.key`. Abweichender Pfad:
  `REMOTEDESK_UPDATER_KEY`. Ein gegebenenfalls benötigtes Kennwort wird über
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` übergeben. Schlüssel und Kennwort dürfen
  weder eingecheckt noch in Ausgaben protokolliert werden.
- Vollständiges, gepatchtes FreeRDP-Backend. Für einen Neubau:
  `npm run build:rdp-backend`.

## Ablauf

1. Version in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`,
   `src-tauri/Cargo.lock` und `src-tauri/tauri.conf.json` setzen; Changelog ergänzen.
2. `npm ci`, `npm run build`, `npm test` und
   `cargo test --manifest-path src-tauri/Cargo.toml` ausführen. RDP-Tests können
   Zugriff auf den macOS-Schlüsselbund benötigen.
3. `npm run tauri:build -- --bundles app` baut und signiert die App samt Backend.
4. `npm run notarize` führt den vollständigen Distributionsablauf aus:
   eingebettetes Backend und äußere App nachsiegeln, App einreichen, Apples
   Annahme prüfen, Ticket anheften und Gatekeeper-Prüfung durchführen.
   Anschließend erzeugt und signiert das Skript das Updater-Archiv und erstellt
   `latest.json`. Zum Schluss baut es aus der notarisierten App ein neues DMG,
   signiert und notarisiert auch dieses und heftet dessen Ticket an.
5. Update-Signatur gegen den in der App konfigurierten öffentlichen Schlüssel
   prüfen. Das Archiv muss die notarisierten, unveränderten App-Dateien enthalten.
6. Den vollständigen Quellstand per Pull Request nach `main` übernehmen und den
   Release-Tag auf den integrierten Commit setzen. Erst danach veröffentlichen.

Die vier Release-Dateien sind:

- `src-tauri/target/release/bundle/updater/RemoteDeskRDP.app.tar.gz`
- `src-tauri/target/release/bundle/updater/RemoteDeskRDP.app.tar.gz.sig`
- `src-tauri/target/release/bundle/updater/latest.json`
- `src-tauri/target/release/bundle/dmg/RemoteDeskRDP_<Version>_aarch64.dmg`

Das GitHub-Release muss als aktuelles Release verfügbar sein, weil bestehende
Installationen das Manifest über
`releases/latest/download/latest.json` abrufen. Build-Ausgaben bleiben außerhalb
der Versionsverwaltung und werden ausschließlich als Release-Dateien hochgeladen.

## Signaturregeln für das Backend

Von innen nach außen signieren. Die Hauptdatei eines App-Bundles nicht einzeln
signieren: `codesign` behandelt sie als Bundle und würde die Hülle zu früh
versiegeln. Auch `--deep` beim Signieren der äußeren App vermeiden; es kann
Metadaten der eingebetteten Bundles überschreiben.

`scripts/sign-freerdp-backend.sh` löst Bibliotheks-Symlinks vor dem Signieren auf
und passt Mach-O-Verweise an. Das verhindert ungültige Bibliotheksnamen, wenn
Tauri Ressourcen kopiert. Die nativen FreeRDP-Programme erhalten ihre eigenen
Entitlements; die äußere App verwendet `src-tauri/Entitlements.plist` beim Bau.

Ein erfolgreicher Upload allein belegt keine Notarisierung. Erforderlich sind
Apples Status `Accepted`, erfolgreiche Ticket-Prüfung und Gatekeepers Ergebnis
`accepted` mit `source=Notarized Developer ID`.
