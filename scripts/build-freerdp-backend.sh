#!/usr/bin/env bash
set -euo pipefail

# Produces the native Cocoa RDP client as a universal app bundle. The upstream
# script builds and embeds its own libraries, plugins and frameworks, so a
# RemoteDeskRDP release has no Homebrew or XQuartz runtime dependency.
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work_root="${project_root}/.build/freerdp"
source_root="${work_root}/FreeRDP"
backend_root="${project_root}/src-tauri/resources/freerdp"
backend_app="${backend_root}/MacFreeRDP.app"
freerdp_ref="${FREERDP_REF:-3.33.1}"
# Seit 3.33 legt bundle-mac-os.sh Quellen, Bau und Ergebnis unter build-macos/
# neben dem Skriptordner ab, nicht mehr im aktuellen Verzeichnis.
build_base="${source_root}/build-macos"

for tool in git cmake ninja autoconf automake aclocal autoheader glibtoolize meson; do
  command -v "${tool}" >/dev/null || {
    echo "Missing build tool: ${tool}" >&2
    exit 1
  }
done

mkdir -p "${work_root}"
if [[ ! -d "${source_root}/.git" ]]; then
  git clone --depth 1 --branch "${freerdp_ref}" https://github.com/FreeRDP/FreeRDP.git "${source_root}"
else
  # Ein vorhandener Checkout kann von einem früheren Lauf mit anderem
  # FREERDP_REF stammen. Ohne Umschalten entstünde still ein Backend aus der
  # falschen Version, auf das der Patch womöglich nur zufällig passt.
  wanted="$(git -C "${source_root}" rev-parse -q --verify "refs/tags/${freerdp_ref}^{commit}" 2>/dev/null || true)"
  current="$(git -C "${source_root}" rev-parse HEAD)"
  if [[ -z "${wanted}" ]]; then
    git -C "${source_root}" fetch --depth 1 origin "${freerdp_ref}"
    git -C "${source_root}" checkout --force --detach "FETCH_HEAD^{commit}"
  elif [[ "${wanted}" != "${current}" ]]; then
    git -C "${source_root}" checkout --force --detach "${wanted}"
  fi
fi
# Optional gegen einen festen Commit prüfen, damit ein umgehängter Tag auffällt.
if [[ -n "${FREERDP_COMMIT:-}" ]]; then
  actual_commit="$(git -C "${source_root}" rev-parse HEAD)"
  if [[ "${actual_commit}" != "${FREERDP_COMMIT}" ]]; then
    echo "FreeRDP checkout ${actual_commit} does not match FREERDP_COMMIT=${FREERDP_COMMIT}" >&2
    exit 1
  fi
fi

(
  cd "${source_root}"
  # bundle-mac-os.sh klont seine Abhaengigkeiten nur, wenn src/ fehlt. Nach einem
  # Versionswechsel blieben sonst die alten Staende (OpenSSL, FFmpeg, SDL ...)
  # liegen und der neue Client wuerde gegen sie gebaut.
  deps_marker="${build_base}/src/.remotedesk-freerdp-ref"
  if [ -d "${build_base}/src" ] && [ "$(cat "${deps_marker}" 2>/dev/null || true)" != "${freerdp_ref}" ]; then
    rm -rf "${build_base}/src"
  fi
  # FreeRDP's macOS bundling script does not enable the Cocoa client by default,
  # and the Cocoa window is pinned to a fixed size. The SDL client presents a
  # frame per update packet, which makes every screen refresh wait for the
  # display several times over. Files cannot be dragged into or out of the
  # session. Measuring the displays through a fullscreen probe window blacks out
  # the whole screen for a moment on macOS.
  # The Cocoa library also fails to link since 3.32 because it compiles
  # client.c without the oauth2.c helpers it now needs.
  # Our patch fixes all of these.
  # Reset the patched files first so a partially reverted tree cannot silently
  # produce an unpatched backend.
  git checkout -- scripts/bundle-mac-os.sh \
    client/Mac/CMakeLists.txt \
    client/Mac/cli/AppDelegate.h client/Mac/cli/AppDelegate.m client/Mac/cli/MainMenu.xib \
    client/SDL/SDL3/sdl_context.cpp client/SDL/SDL3/sdl_context.hpp \
    client/SDL/SDL3/sdl_freerdp.cpp client/SDL/SDL3/sdl_clip.cpp client/SDL/SDL3/sdl_clip.hpp \
    client/SDL/SDL3/sdl_window.cpp \
    client/common/client_cliprdr_file.c include/freerdp/client/client_cliprdr_file.h
  git apply "${project_root}/scripts/patches/freerdp-3.33.1-macos-client.patch"
  # SDL only compares the macOS clipboard when a window gains focus, so nothing
  # copied elsewhere ever reaches the session. bundle-mac-os.sh applies this
  # patch after cloning, so a fresh source tree is patched as well, and resets
  # an existing SDL checkout first.
  # pkg-config sucht sonst auch unter /opt/homebrew. CMAKE_IGNORE_PATH im
  # Bundling-Skript greift dort nicht: Ist Homebrew-FFmpeg installiert, linkt
  # libfreerdp3 dessen libavcodec (MJPEG-Zweig in libfreerdp/CMakeLists.txt),
  # und das Backend startet auf keinem anderen Mac. CMake ergaenzt die eigenen
  # Abhaengigkeiten ueber CMAKE_PREFIX_PATH selbst.
  PKG_CONFIG_LIBDIR="${build_base}/install/MacFreeRDP.app/Contents/lib/pkgconfig" \
  REMOTEDESK_SDL_PATCH="${project_root}/scripts/patches/sdl-3.2.28-clipboard-poll.patch" \
    ./scripts/bundle-mac-os.sh --arch "arm64 x86_64" --target "12"
  echo "${freerdp_ref}" > "${deps_marker}"
)

source_app="${build_base}/install/MacFreeRDP.app"
test -x "${source_app}/Contents/MacOS/MacFreeRDP" || {
  echo "FreeRDP did not produce MacFreeRDP.app" >&2
  exit 1
}
test -x "${source_app}/Contents/MacOS/sdl-freerdp" || {
  echo "FreeRDP did not produce the SDL client" >&2
  exit 1
}

# The upstream script creates the framework-style directory layout but leaves
# out the Cocoa application metadata when built with Ninja.  MacFreeRDP uses
# NSApplicationMain, so both files are required for it to create its window.
cp "${project_root}/scripts/MacFreeRDP-Info.plist" "${source_app}/Contents/Info.plist"
# Das Sitzungsfenster ist fuer den Nutzer Teil von RemoteDeskRDP, also traegt
# es auch dessen Symbol.  FreeRDP.icns bleibt liegen, damit der Patchstand
# gegenueber dem Original nachvollziehbar bleibt.
cp "${project_root}/src-tauri/icons/icon.icns" "${source_app}/Contents/Resources/RemoteDeskRDP.icns"
cp "${build_base}/build/freerdp/client/Mac/cli/MainMenu.nib" "${source_app}/Contents/Resources/MainMenu.nib"
cp "${build_base}/build/freerdp/client/Mac/CertificateDialog.nib" "${source_app}/Contents/Resources/CertificateDialog.nib"
cp "${build_base}/build/freerdp/client/Mac/PasswordDialog.nib" "${source_app}/Contents/Resources/PasswordDialog.nib"

rm -rf "${backend_app}"
mkdir -p "${backend_root}"
ditto "${source_app}" "${backend_app}"
cp "${source_root}/LICENSE" "${backend_root}/FREERDP-LICENSE.txt"

# Apache-2.0, LGPL, BSD, zlib und die Fraunhofer-Lizenz verlangen alle, dass
# ihr vollstaendiger Text mit dem Binaerpaket ausgeliefert wird.  Die Namen
# links entsprechen den Eintraegen im Lizenzfenster der App.
licenses_root="${backend_root}/licenses"
rm -rf "${licenses_root}"
mkdir -p "${licenses_root}"
copy_license() {
  local target="$1" source="${build_base}/src/$2"
  if [ -f "${source}" ]; then
    cp "${source}" "${licenses_root}/${target}"
  else
    echo "Lizenztext fehlt: ${source}" >&2
    return 1
  fi
}
copy_license "FFmpeg-LICENSE.md"     "FFmpeg/LICENSE.md"
copy_license "FFmpeg-COPYING.LGPLv2.1" "FFmpeg/COPYING.LGPLv2.1"
copy_license "SDL-LICENSE.txt"       "SDL/LICENSE.txt"
copy_license "SDL_image-LICENSE.txt" "SDL_image/LICENSE.txt"
copy_license "SDL_ttf-LICENSE.txt"   "SDL_ttf/LICENSE.txt"
copy_license "OpenSSL-LICENSE.txt"   "openssl/LICENSE.txt"
copy_license "libusb-LICENSE"        "libusb-cmake/LICENSE"
copy_license "openh264-LICENSE"      "openh264/LICENSE"
copy_license "opus-COPYING"          "opus/COPYING"
copy_license "uriparser-COPYING.BSD-3-Clause" "uriparser/COPYING.BSD-3-Clause"
copy_license "json-c-COPYING"        "json-c/COPYING"
copy_license "zlib-LICENSE"          "zlib/LICENSE"
copy_license "fdk-aac-NOTICE"        "fdk-aac/NOTICE"

# Sicherheitsnetz: Kein Mach-O im Bundle darf auf Bibliotheken des Build-Macs
# (Homebrew, MacPorts, /usr/local) verweisen, sonst startet das Backend
# nur hier.
foreign_links=""
while IFS= read -r -d '' candidate; do
  if file "${candidate}" | grep -q 'Mach-O'; then
    hits="$(otool -L "${candidate}" 2>/dev/null | grep -E '^[[:space:]]+(/opt/homebrew|/opt/local|/usr/local)/' || true)"
    if [[ -n "${hits}" ]]; then
      foreign_links+="${candidate}:"$'\n'"${hits}"$'\n'
    fi
  fi
done < <(find "${backend_app}/Contents" -type f -print0)
if [[ -n "${foreign_links}" ]]; then
  echo "Fremde Bibliothekspfade im Backend gefunden:" >&2
  printf '%s' "${foreign_links}" >&2
  exit 1
fi

echo "Bundled backend: ${backend_app}"
echo "Bundled licenses: $(ls -1 "${licenses_root}" | wc -l | tr -d ' ') Dateien"

# Ohne Signatur scheitert spaeter die Notarisierung. Gleich hier erledigen,
# damit ein frisch gebautes Backend nie unsigniert liegen bleibt.
"${script_dir:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}/sign-freerdp-backend.sh" "${backend_app}"
