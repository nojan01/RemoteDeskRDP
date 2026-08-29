#!/usr/bin/env bash
#
# Signiert das mitgelieferte FreeRDP-Backend mit der Developer ID.
#
# Warum eigenes Skript: Tauri signiert nur die aussere App und die eigenen
# Binaerdateien. Alles unter `resources/` reicht es unveraendert durch. Ohne
# diesen Schritt bleiben rund 35 Mach-O-Dateien unsigniert und die
# Notarisierung schlaegt fehl ("The binary is not signed with a valid
# Developer ID certificate").
#
# Der Schritt gehoert vor `tauri build` - die Signaturen stecken in den
# Dateien selbst und ueberleben das Kopieren ins App-Bundle.
#
# Aufruf:
#   scripts/sign-freerdp-backend.sh [pfad-zum-MacFreeRDP.app]

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
backend_app="${1:-${script_dir}/../src-tauri/resources/freerdp/MacFreeRDP.app}"

if [ ! -d "${backend_app}" ]; then
  echo "Backend nicht gefunden: ${backend_app}" >&2
  exit 1
fi
backend_app="$(cd "${backend_app}" && pwd)"

identity="${APPLE_SIGNING_IDENTITY:-Developer ID Application: Norbert Jander (TXF2V79Z6N)}"

if ! security find-identity -v -p codesigning | grep -qF "${identity}"; then
  echo "Signatur-Identitaet nicht im Schluesselbund: ${identity}" >&2
  echo "Vorhanden:" >&2
  security find-identity -v -p codesigning >&2
  exit 1
fi

echo "Identitaet: ${identity}"
echo "Backend:    ${backend_app}"

# --options runtime  = Hardened Runtime, von der Notarisierung verlangt
# --timestamp        = sicherer Zeitstempel von Apple, ebenfalls Pflicht
# --force            = vorhandene Signaturen ersetzen (Neubau des Backends)
sign_flags=(--force --options runtime --timestamp --sign "${identity}")

# Reihenfolge zaehlt: von innen nach aussen. Wird die Huelle zuerst signiert,
# entwertet jede spaetere Signatur im Inneren das aeussere Siegel.
#
# Nur echte Dateien: `-type f` laesst die 34 Symlinks (libz.dylib ->
# libz.1.4.1.1.dylib und aehnliche) aus. Symlinks tragen keine Signatur, ein
# Signierversuch bricht ab.
# Nach Pfadlaenge absteigend sortiert - das setzt tiefer liegende Dateien
# zuverlaessig nach vorn, ohne die Ordnerstruktur kennen zu muessen.
#
# `mapfile` gibt es nicht: macOS liefert Bash 3.2 aus.
# Achtung, teuer erkauft: Die Hauptdatei des Bundles
# (`Contents/MacOS/<CFBundleExecutable>`) darf **nicht** einzeln signiert
# werden. codesign erkennt sie als Bundle-Hauptdatei und signiert daraufhin das
# gesamte Bundle - noch bevor die uebrigen Dateien an der Reihe waren. Es bricht
# dann mit "code object is not signed at all / In subcomponent: ..." ab. Sie
# wird ausschliesslich im abschliessenden Bundle-Schritt signiert.
main_executable="$(
  /usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' \
    "${backend_app}/Contents/Info.plist" 2>/dev/null || echo ''
)"
main_executable_path="${backend_app}/Contents/MacOS/${main_executable}"

macho_files=()
while IFS= read -r file; do
  [ -z "${file}" ] && continue
  [ "${file}" = "${main_executable_path}" ] && continue
  macho_files+=("${file}")
done < <(
    find "${backend_app}" -type f -print0 \
    | xargs -0 file 2>/dev/null \
    | sed 's/ (for architecture [^)]*)//' \
    | grep 'Mach-O' \
    | cut -d: -f1 \
    | sort -u \
    | awk '{ print length"\t"$0 }' \
    | sort -rn \
    | cut -f2-
)

if [ "${#macho_files[@]}" -eq 0 ]; then
  echo "Keine Mach-O-Dateien gefunden - das kann nicht stimmen." >&2
  exit 1
fi

echo "Signiere ${#macho_files[@]} Mach-O-Dateien ..."
for file in "${macho_files[@]}"; do
  codesign "${sign_flags[@]}" "${file}"
done

# Zuletzt die Huelle: versiegelt Info.plist und Resources.
echo "Signiere das Bundle ..."
codesign "${sign_flags[@]}" "${backend_app}"

echo "Pruefe ..."
codesign --verify --deep --strict --verbose=2 "${backend_app}"

unsigned=0
for file in "${macho_files[@]}"; do
  if ! codesign --verify --strict "${file}" 2>/dev/null; then
    echo "NICHT signiert: ${file}" >&2
    unsigned=$((unsigned + 1))
  fi
done

if [ "${unsigned}" -gt 0 ]; then
  echo "${unsigned} Datei(en) ohne gueltige Signatur." >&2
  exit 1
fi

echo "Fertig: ${#macho_files[@]} Dateien + Bundle signiert, Hardened Runtime aktiv."
