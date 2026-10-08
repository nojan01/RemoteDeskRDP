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

# Symlinks aufloesen, bevor signiert wird.
#
# Tauri kopiert `resources/` ohne Symlinks: aus `libfreerdp-client3.3.dylib ->
# libfreerdp-client3.3.26.0.dylib` wird eine zweite echte Datei. Deren
# Install-Name lautet weiter `libfreerdp-client3.3.26.0.dylib`. dyld verlangt
# bei Hardened Runtime aber, dass der Dateiname zum angefragten Namen passt,
# und bricht mit "Library missing" (SIGABRT beim Start) ab.
#
# Deshalb zeigen alle Mach-O-Verweise danach direkt auf die echte Datei, und
# die Symlinks werden entfernt.
link_names=()
link_targets=()
while IFS= read -r link; do
  [ -z "${link}" ] && continue
  target="${link}"
  while [ -L "${target}" ]; do
    next="$(readlink "${target}")"
    case "${next}" in
      /*) target="${next}" ;;
      *) target="$(dirname "${target}")/${next}" ;;
    esac
  done
  link_names+=("$(basename "${link}")")
  link_targets+=("$(basename "${target}")")
done < <(find "${backend_app}" -type l -name '*.dylib')

if [ "${#link_names[@]}" -gt 0 ]; then
  echo "Loese ${#link_names[@]} Symlinks auf ..."
  while IFS= read -r file; do
    [ -z "${file}" ] && continue
    changes=()
    while IFS= read -r dep; do
      dep_name="${dep##*/}"
      i=0
      while [ "${i}" -lt "${#link_names[@]}" ]; do
        if [ "${link_names[${i}]}" = "${dep_name}" ]; then
          changes+=(-change "${dep}" "${dep%/*}/${link_targets[${i}]}")
          break
        fi
        i=$((i + 1))
      done
    done < <(otool -L "${file}" | tail -n +2 | awk '{ print $1 }' | grep '/' | sort -u)
    if [ "${#changes[@]}" -gt 0 ]; then
      install_name_tool "${changes[@]}" "${file}" 2>/dev/null
    fi
  done < <(
    find "${backend_app}" -type f -print0 \
    | xargs -0 file 2>/dev/null \
    | grep 'Mach-O' \
    | cut -d: -f1 \
    | sort -u
  )
  find "${backend_app}" -type l -name '*.dylib' -delete
fi

# Jeder @rpath-Verweis muss jetzt auf eine echte Datei in Frameworks zeigen.
frameworks_dir="${backend_app}/Contents/Frameworks"
missing=0
while IFS= read -r dep; do
  [ -z "${dep}" ] && continue
  if [ ! -f "${frameworks_dir}/${dep#@rpath/}" ]; then
    echo "Fehlende Library: ${dep}" >&2
    missing=$((missing + 1))
  fi
done < <(
  find "${backend_app}" -type f -print0 \
  | xargs -0 file 2>/dev/null \
  | grep 'Mach-O' \
  | cut -d: -f1 \
  | sort -u \
  | while IFS= read -r f; do otool -L "${f}" | tail -n +2 | awk '{ print $1 }'; done \
  | grep '^@rpath/' \
  | sort -u
)
if [ "${missing}" -gt 0 ]; then
  echo "${missing} Verweis(e) ohne passende Datei." >&2
  exit 1
fi

# --options runtime  = Hardened Runtime, von der Notarisierung verlangt
# --timestamp        = sicherer Zeitstempel von Apple, ebenfalls Pflicht
# --force            = vorhandene Signaturen ersetzen (Neubau des Backends)
sign_flags=(--force --options runtime --timestamp --sign "${identity}")
# Die beiden RDP-Clients duerfen das Mikrofon nutzen (/microphone). Die
# Berechtigung steckt in der Signatur und gilt so auch im Startbundle je Sitzung.
entitlements="${script_dir}/freerdp-client.entitlements"
client_flags=("${sign_flags[@]}" --entitlements "${entitlements}")

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
  case "${file##*/}" in
    sdl-freerdp) codesign "${client_flags[@]}" "${file}" ;;
    *) codesign "${sign_flags[@]}" "${file}" ;;
  esac
done

# Zuletzt die Huelle: versiegelt Info.plist und Resources.
echo "Signiere das Bundle ..."
codesign "${client_flags[@]}" "${backend_app}"

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
