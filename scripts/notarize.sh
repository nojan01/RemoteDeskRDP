#!/usr/bin/env bash
#
# Notarisiert RemoteDeskRDP: siegelt nach, reicht bei Apple ein, heftet das
# Ticket an und weist das Ergebnis nach.
#
# Warum ueberhaupt nachsiegeln?
#
#   Tauri kopiert alles unter `resources/` mit aufgeloesten Symlinks. Aus den
#   34 Symlinks des FreeRDP-Backends (libz.1.dylib -> libz.1.4.1.1.dylib und
#   aehnliche) werden dabei echte Dateien. Das eingebettete MacFreeRDP.app ist
#   aber ein eigenes Bundle mit eigenem Siegel - und dieses Siegel haelt die
#   Symlinks fest. Nach dem Kopieren passt es nicht mehr:
#
#       codesign --verify ...  ->  "file modified: .../libz.1.dylib"
#
#   Die aeussere App verifiziert trotzdem sauber, weil ihr Siegel nur die
#   Dateiinhalte umfasst. Apples Notardienst prueft jedoch genauer und lehnt ab:
#
#       "The signature of the binary is invalid"
#       .../MacFreeRDP.app/Contents/MacOS/MacFreeRDP
#
#   Gemessen am 01.08.2026, Vorgang 4f1fd1ef-1862-45c1-ab39-8c549e12c31d.
#   Nach dem Nachsiegeln wurde derselbe Bau angenommen (bb133bd5-...).
#
# Ablauf:  tauri build  ->  dieses Skript  ->  App nach /Applications
#
# Aufruf:
#   scripts/notarize.sh [pfad-zu-RemoteDeskRDP.app]

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
app="${1:-${script_dir}/../src-tauri/target/release/bundle/macos/RemoteDeskRDP.app}"
profile="${NOTARY_PROFILE:-remotedesk-notary}"
identity="${APPLE_SIGNING_IDENTITY:-Developer ID Application: Norbert Jander (TXF2V79Z6N)}"

if [ ! -d "${app}" ]; then
  echo "App nicht gefunden: ${app}" >&2
  echo "Zuerst bauen: npm run tauri:build" >&2
  exit 1
fi
app="$(cd "${app}" && pwd)"

if ! xcrun notarytool history --keychain-profile "${profile}" >/dev/null 2>&1; then
  echo "Kein Notar-Profil \"${profile}\" im Schluesselbund." >&2
  echo "Anlegen mit:" >&2
  echo "  xcrun notarytool store-credentials \"${profile}\" --team-id TXF2V79Z6N" >&2
  exit 1
fi

backend="${app}/Contents/Resources/resources/freerdp/MacFreeRDP.app"

echo "== 1/5  Eingebettetes FreeRDP-Bundle nachsiegeln =="
if [ -d "${backend}" ]; then
  "${script_dir}/sign-freerdp-backend.sh" "${backend}" >/dev/null
  echo "   ${backend##*/Resources/} neu gesiegelt"
else
  echo "   Kein eingebettetes Backend gefunden - uebersprungen." >&2
fi

# Ohne --deep: das wuerde die eingebetteten Bundles mit der Kennung der
# aeusseren App ueberschreiben und das gerade erneuerte Siegel wieder zerstoeren.
echo "== 2/5  Aeussere App neu signieren =="
codesign --force --options runtime --timestamp --sign "${identity}" "${app}"
codesign --verify --deep --strict "${app}"
codesign --verify --deep --strict "${backend}"

echo "== 3/5  Einreichen =="
archive="$(mktemp -d)/$(basename "${app}" .app).zip"
/usr/bin/ditto -c -k --keepParent "${app}" "${archive}"

submit_log="$(mktemp)"
if ! xcrun notarytool submit "${archive}" \
       --keychain-profile "${profile}" --wait 2>&1 | tee "${submit_log}"; then
  echo "Einreichen fehlgeschlagen." >&2
  exit 1
fi

if ! grep -q "status: Accepted" "${submit_log}"; then
  echo >&2
  echo "Apple hat abgelehnt. Begruendung:" >&2
  submission_id="$(grep -m1 '  id: ' "${submit_log}" | awk '{print $2}')"
  detail="$(mktemp).json"
  xcrun notarytool log "${submission_id}" --keychain-profile "${profile}" "${detail}" >/dev/null 2>&1 || true
  python3 - "${detail}" <<'PY' >&2 || cat "${detail}" >&2
import json, sys, collections
d = json.load(open(sys.argv[1]))
for (sev, msg), n in collections.Counter(
        (i.get("severity"), i.get("message")) for i in d.get("issues") or []).most_common():
    print(f"  [{sev}] {n}x  {msg}")
for i in (d.get("issues") or [])[:10]:
    print("   -", i.get("path"))
PY
  exit 1
fi

echo "== 4/5  Ticket anheften =="
xcrun stapler staple "${app}"

echo "== 5/5  Nachweis =="
# "accepted / source=Notarized Developer ID" ist der einzige Beleg, der zaehlt.
spctl -a -vvv -t exec "${app}"
xcrun stapler validate "${app}"

rm -rf "$(dirname "${archive}")" "${submit_log}"
echo
echo "Fertig. Installieren mit:"
echo "  rm -rf /Applications/$(basename "${app}") && ditto \"${app}\" \"/Applications/$(basename "${app}")\""
