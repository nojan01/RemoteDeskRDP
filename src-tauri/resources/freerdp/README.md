# FreeRDP release resource

The distributable RemoteDesk app bundles FreeRDP's universal, signed Cocoa
client at this location as `MacFreeRDP`, together with all required dynamic libraries and
plugins. It must be built for both `arm64` and `x86_64`, have relative install
names, and be signed before the parent app is notarized.

This source repository intentionally does not commit a machine-specific binary.
Development uses `REMOTEDESK_RDP_EXECUTABLE` or a locally installed `MacFreeRDP`
binary. `xfreerdp` is deliberately not used as a fallback: it is often an
X11/SDL client and does not open a native macOS session window. A release
pipeline replaces this document with the packaged backend and its Apache-2.0
license notices.
