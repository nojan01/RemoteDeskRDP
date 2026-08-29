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
freerdp_ref="${FREERDP_REF:-3.26.0}"

for tool in git cmake ninja autoconf automake aclocal autoheader glibtoolize meson; do
  command -v "${tool}" >/dev/null || {
    echo "Missing build tool: ${tool}" >&2
    exit 1
  }
done

mkdir -p "${work_root}"
if [[ ! -d "${source_root}/.git" ]]; then
  git clone --depth 1 --branch "${freerdp_ref}" https://github.com/FreeRDP/FreeRDP.git "${source_root}"
fi

(
  cd "${source_root}"
  # FreeRDP's macOS bundling script does not enable the Cocoa client by default,
  # and the Cocoa window is pinned to a fixed size. The SDL client presents a
  # frame per update packet, which makes every screen refresh wait for the
  # display several times over, and it announces an empty clipboard format list
  # while negotiating the channel. Copying files out of the session is gated
  # behind FUSE, which macOS cannot provide. Its textures also keep SDL's
  # default alpha blending, which blanks the image at 24 and 32 bpp.
  # Our patch fixes all of these.
  # Reset the patched files first so a partially reverted tree cannot silently
  # produce an unpatched backend.
  git checkout -- scripts/bundle-mac-os.sh client/Mac/Keyboard.m \
    client/Mac/cli/AppDelegate.h client/Mac/cli/AppDelegate.m client/Mac/cli/MainMenu.xib \
    client/SDL/SDL3/sdl_context.cpp client/SDL/SDL3/sdl_context.hpp \
    client/SDL/SDL3/sdl_freerdp.cpp client/SDL/SDL3/sdl_clip.cpp client/SDL/SDL3/sdl_clip.hpp \
    client/SDL/SDL3/sdl_window.cpp \
    client/common/client_cliprdr_file.c include/freerdp/client/client_cliprdr_file.h
  git apply "${project_root}/scripts/patches/freerdp-3.26.0-macos-client.patch"
  # SDL only compares the macOS clipboard when a window gains focus, so nothing
  # copied elsewhere ever reaches the session, and it never exposes copied files
  # as text/uri-list. bundle-mac-os.sh applies this patch after cloning, so a
  # fresh source tree is patched as well. Reset an existing checkout first, or
  # applying the patch a second time fails and aborts the build.
  if [ -d src/SDL/.git ]; then
    git -C src/SDL checkout -- .
  fi
  REMOTEDESK_SDL_PATCH="${project_root}/scripts/patches/sdl-3.2.28-clipboard-poll.patch" \
    ./scripts/bundle-mac-os.sh --arch "arm64 x86_64" --target "12"
)

source_app="${source_root}/install/MacFreeRDP.app"
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
cp "${source_root}/build/freerdp/client/Mac/cli/MainMenu.nib" "${source_app}/Contents/Resources/MainMenu.nib"
cp "${source_root}/build/freerdp/client/Mac/CertificateDialog.nib" "${source_app}/Contents/Resources/CertificateDialog.nib"
cp "${source_root}/build/freerdp/client/Mac/PasswordDialog.nib" "${source_app}/Contents/Resources/PasswordDialog.nib"

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
  local target="$1" source="${source_root}/src/$2"
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

echo "Bundled backend: ${backend_app}"
echo "Bundled licenses: $(ls -1 "${licenses_root}" | wc -l | tr -d ' ') Dateien"

# Ohne Signatur scheitert spaeter die Notarisierung. Gleich hier erledigen,
# damit ein frisch gebautes Backend nie unsigniert liegen bleibt.
"${script_dir:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}/sign-freerdp-backend.sh" "${backend_app}"
