// English texts. Keys must mirror de.ts – the test in i18n.test.ts enforces it.

export const en: Record<string, string> = {
  // ─── Sidebar ────────────────────────────────────────────────────
  "brand.subtitle": "RDP · VNC · SSH",
  "side.connections": "CONNECTIONS",
  "side.notConfigured": "not configured",
  "side.empty": "No saved connections yet.",
  "side.newProfile": "＋ New connection",
  "side.help": "ⓘ Help & compatibility",
  "side.themeTitle": "Appearance: {label}",
  "side.langTitle": "Language: {label}",
  "theme.auto": "Auto (follows macOS)",
  "theme.light": "Light",
  "theme.dark": "Dark",
  "lang.auto": "Auto (follows macOS)",
  "lang.de": "German",
  "lang.en": "English",

  // ─── Header ─────────────────────────────────────────────────────
  "top.eyebrow": "CONNECTION PROFILE",
  "top.untitled": "New connection",

  // ─── Status messages ────────────────────────────────────────────
  "state.ready": "Ready for a secure connection",
  "state.loadFailed": "Could not load profiles: {error}",
  "state.profileLoaded": "Profile loaded",
  "state.newProfile": "Creating a new connection",
  "state.passwordCleared": "Session password removed from the keychain.",
  "state.passwordClearFailed": "Could not remove the session password: {error}",
  "state.gatewayPasswordCleared": "Gateway password removed from the keychain.",
  "state.gatewayPasswordClearFailed": "Could not remove the gateway password: {error}",
  "state.saved": "Profile saved securely",
  "state.saveFailed": "Saving failed: {error}",
  "state.deleted": "Profile deleted",
  "state.deleteFailed": "Deleting failed: {error}",
  "state.connecting": "Preparing the connection …",
  "state.linkUnknown": "The requested connection is not known here.",
  "state.connected": "{name} was started in its own session window.",
  "state.vncConnected": "{name} is connected in this VNC session.",
  "state.vncWindowOpened": "The VNC session for {name} was opened in its own window.",
  "state.vncClosed": "VNC session was closed.",
  "state.connectFailed": "Connection failed: {error}",
  "state.folderTwice": "That folder is already shared",
  "confirm.delete": "Really delete this connection profile?",

  // ─── Identity ───────────────────────────────────────────────────
  "field.name": "Display name",
  "field.namePlaceholder": "e.g. Office PC",
  "field.protocol": "Protocol",
  "field.host": "Host or IP address",
  "field.hostPlaceholder": "server.example.net or 192.168.1.20",
  "field.username": "User name",
  "field.domain": "Domain",
  "field.optional": "optional",
  "field.password": "Password",
  "field.passwordPlaceholder": "Store in the macOS keychain",
  "field.passwordNote": "Never stored in the profile or passed as a process argument.",
  "protocol.rdp": "RDP (Remote Desktop)",
  "protocol.vnc": "VNC (integrated session)",
  "protocol.ssh": "SSH (terminal)",
  "protocol.sftp": "SFTP/SCP (file terminal)",
  "protocol.mosh": "Mosh (terminal)",
  "protocol.s3": "S3 (object storage)",
  "protocol.swift": "OpenStack Swift (object storage)",
  "action.reveal": "Show",
  "action.hide": "Hide",
  "action.remove": "Remove",

  // ─── Object storage ─────────────────────────────────────────────
  "object.eyebrow": "OBJECT STORAGE",
  "object.s3Title": "S3 connection",
  "object.swiftTitle": "OpenStack Swift connection",
  "object.keychain": "Secret in keychain",
  "object.s3Endpoint": "S3 endpoint URL",
  "object.swiftEndpoint": "Swift location / endpoint",
  "object.accessKey": "Access key",
  "object.secretKey": "Secret access key",
  "object.swiftPassword": "Swift password",
  "object.region": "AWS / OpenStack region",
  "object.container": "Starting bucket / container",
  "object.containerOptional": "empty = all buckets / containers",
  "object.containerNote": "Optional. Any other bucket or container can be selected in the session window.",
  "object.swiftProject": "Keystone project",
  "object.swiftTenant": "Keystone tenant",
  "object.swiftVersion": "Keystone version",
  "object.swiftVersionV3": "Keystone v3 — current",
  "object.swiftVersionV2": "Keystone v2 — legacy",
  "object.swiftVersionNote": "v3 is the current standard. v2 is intended only for older clouds.",
  "object.swiftPath": "Keystone path",
  "object.swiftPathNote": "Provider-specific path, for example /identity/v3 at Infomaniak.",
  "object.swiftUserDomain": "User domain",
  "object.swiftProjectDomain": "Project domain",
  "object.secretPlaceholder": "Store in the macOS keychain",
  "object.secretNote": "Never stored in profiles.json or passed to an external process.",
  "object.pathStyle": "Use S3 path style (for MinIO and many compatible providers)",
  "object.s3Note": "Supports AWS S3 and S3-compatible providers. The app signs every request locally with AWS Signature Version 4.",
  "object.swiftNote": "Supports Keystone v3 and v2 for older clouds, and refreshes the access token for each load. The Keystone path is configurable. Region may stay empty if there is only one object-storage endpoint.",
  "object.close": "Close object storage",
  "object.refresh": "Refresh",
  "object.up": "Up one level",
  "object.upload": "Upload",
  "object.uploadPick": "Choose file to upload",
  "object.download": "Download",
  "object.downloadPick": "Choose destination folder",
  "object.uploaded": "{name} was uploaded.",
  "object.downloaded": "Saved: {path}",
  "object.containers": "Buckets and containers",
  "object.folder": "Prefix",
  "object.empty": "No entries found.",
  "confirm.objectDelete": "Really delete “{name}” from object storage?",

  // ─── Transport ──────────────────────────────────────────────────
  "transport.eyebrow": "TRANSPORT",
  "transport.title": "Port & network",
  "transport.fallback": "↯ TCP fallback active",
  "transport.udpPreferred": "Prefer UDP",
  "transport.autoReconnect": "Reconnect",
  "transport.note": "UDP is used automatically when the target server and the firewall allow it. Direct FreeRDP connections use the same target port as RDP TCP; if the port values differ, the TCP port applies. Otherwise the session stays on TCP.",
  "vnc.note": "The VNC session is shown directly in RemoteDeskRDP. Its password is requested when connecting and is not stored in the profile.",
  "vnc.disconnect": "Close VNC session",
  "vnc.passwordPrompt": "VNC password for {name}:",
  "vnc.passwordRequired": "VNC password required",
  "vnc.passwordHelp": "Enter the password for {name}.",
  "vnc.cancel": "Cancel",
  "vnc.savePassword": "Save password in the macOS keychain",
  "ssh.terminal": "Terminal type",
  "ssh.terminalNote": "Passed to the SSH server as TERM. Default: xterm-256color.",
  "ssh.terminal.xterm256": "xterm-256color — default",
  "ssh.terminal.xterm": "xterm — broad compatibility",
  "ssh.terminal.screen": "screen-256color — GNU Screen",
  "ssh.terminal.tmux": "tmux-256color — tmux",
  "ssh.terminal.vt100": "vt100 — very old systems",
  "ssh.terminal.linux": "linux — local Linux console",
  "ssh.disconnect": "Close SSH session",
  "ssh.x11.enable": "Enable X11 forwarding (-X)",
  "ssh.x11.command": "Start Linux application automatically",
  "ssh.x11.commandPlaceholder": "e.g. gedit or xterm -e htop",
  "ssh.x11.commandNote": "Optional command on the Linux host. It runs after sign-in; closing the application ends the SSH session.",
  "ssh.x11.closeXquartz": "Close XQuartz when the application ends",
  "ssh.x11.note": "Only applies to graphical Linux/Unix applications, not Windows programs. Requires XQuartz on the Mac. If it is missing, install it first from xquartz.org; RemoteDeskRDP starts XQuartz automatically when connecting. The optional shutdown only closes XQuartz when no other RemoteDeskRDP X11 session is active; other XQuartz applications will close as well.",
  "terminal.port": "SSH port",
  "terminal.error": "Terminal error",
  "sftp.note": "Opens the integrated SFTP file terminal over SSH. Use get and put to download and upload securely; key and password prompts appear directly in the window.",
  "mosh.note": "Mosh signs in over SSH and then uses UDP (60000–61000 by default). Mosh must be installed locally; for example with Homebrew: brew install mosh.",
  "unit.port": "PORT",
  "unit.px": "PX",

  // ─── Gateway ────────────────────────────────────────────────────
  "gateway.eyebrow": "GATEWAY",
  "gateway.title": "RD Gateway",
  "gateway.enable": "Connect through a gateway",
  "gateway.host": "Gateway host",
  "gateway.hostNote": "No port and no protocol – just the name.",
  "gateway.port": "Gateway port",
  "gateway.username": "Gateway user",
  "gateway.domain": "Gateway domain",
  "gateway.password": "Gateway password",
  "gateway.sameAsSession": "same as session",
  "gateway.passwordNote": "Separate entry in the macOS keychain, apart from the session password.",
  "gateway.note": "If user, domain and password stay empty, RemoteDeskRDP signs in to the gateway with the session credentials. That is the usual case; separate details are only needed when the gateway requires a different account.",

  // ─── Display ────────────────────────────────────────────────────
  "display.eyebrow": "DISPLAY",
  "display.title": "Window & resolution",
  "display.mode": "Window mode",
  "display.mode.window": "Window with a fixed initial resolution",
  "display.mode.workarea": "Fill the usable screen area",
  "display.mode.fullscreen": "Full screen (⌃⌥⏎ toggles)",
  "display.width": "Width",
  "display.height": "Height",
  "display.resize": "Behaviour when enlarging",
  "display.resize.dynamic": "Follow with the resolution (server must support it)",
  "display.resize.scale": "Scale the image to the window (always possible)",
  "display.resize.fixed": "Fixed resolution, window not resizable",
  "display.depth": "Colour depth",
  "display.depth.auto": "Negotiate automatically",
  "display.depth.32": "32 bit – true colour with alpha channel",
  "display.depth.24": "24 bit – true colour",
  "display.depth.16": "16 bit – high colour, noticeably less data",
  "display.depth.15": "15 bit – high colour",
  "display.depth.8": "8 bit – 256 colours, for very slow links",
  "display.depthNote": "A lower colour depth transfers less data and helps on narrow links. On a local network “Automatic” is the right choice.",
  "display.resizeNote": "“Follow with the resolution” changes the server resolution while enlarging and needs the display control channel. If the server does not offer it, “Scale the image to the window” gives you a freely resizable window.",

  // ─── Session ────────────────────────────────────────────────────
  "session.eyebrow": "SESSION",
  "session.title": "Working environment",
  "session.keychain": "⌘ Keychain",
  "session.clipboard": "Share the clipboard",
  "session.audio": "Forward audio",
  "session.printer": "Share printers",
  "session.smartcard": "Share smart cards",
  "session.video": "Accelerate video playback",
  "session.shares": "Shared folders",
  "session.addFolder": "Add folder",
  "session.pickFolder": "Share a folder with the session",
  "session.shareFallbackName": "Share",
  "session.noShares": "No folder shared. Shares appear in the guest system among the redirected drives.",
  "session.shareName": "Share name",
  "session.removeShare": "Remove share",
  "session.remove": "Remove",
  "session.certificates": "Certificates",
  "session.cert.prompt": "Reject unknown certificates",
  "session.cert.tofu": "Trust on first contact",
  "session.cert.ignore": "Always ignore (insecure)",

  // ─── Footer ─────────────────────────────────────────────────────
  "action.deleteProfile": "Delete profile",
  "action.saveProfile": "Save profile",
  "action.connect": "Connect",

  // ─── Help ───────────────────────────────────────────────────────
  "help.eyebrow": "REMOTEDESKRDP",
  "help.title": "Help & compatibility",
  "help.close": "Close",

  "help.targets.h": "Supported targets",
  "help.targets.body": "<p><b>Windows</b> over RDP or VNC; <b>Linux</b> over RDP with xrdp or VNC; <b>macOS</b> through Screen Sharing over VNC. RemoteDeskRDP shows VNC sessions directly in the app window.</p>",

  "help.ports.h": "Ports",
  "help.ports.body": "<p>RDP TCP and UDP: 3389. Both values can be adjusted per profile.</p>",
  "help.vnc.h": "VNC session",
  "help.vnc.body": "<p>VNC is shown directly in the RemoteDeskRDP window. The integrated viewer speaks standard RFB and therefore works with common VNC servers, including TightVNC, without a server-specific path. The default port is 5900. The password is requested only while connecting and is not stored in the profile. The connection between the display and the app uses a short-lived loopback connection only. RDP options such as gateway, redirected drives or dynamic resolution intentionally stay hidden for VNC.</p>",
  "help.ssh.h": "SSH terminal",
  "help.ssh.body": "<p>SSH opens an interactive terminal in its own RemoteDeskRDP window. The app uses the macOS system client <code>/usr/bin/ssh</code> with a pseudo-terminal, so password, key and host-key prompts appear directly in the terminal. The default port is 22. Keep the user name, port and terminal type in the profile. The terminal type is passed to the server as <code>TERM</code>; <code>xterm-256color</code> is the suitable default for the embedded terminal.</p><p>The optional X11 setting enables <code>-X</code>. It only applies to graphical Linux/Unix applications, not Windows programs. It requires <b>XQuartz</b> on the Mac; RemoteDeskRDP starts it when connecting and passes its display to SSH. A saved Linux command such as <code>gedit</code> or <code>xterm -e htop</code> starts automatically and opens in an XQuartz window on the Mac.</p>",
  "help.sftp.h": "SFTP/SCP file terminal",
  "help.sftp.body": "<p>SFTP opens a secure interactive file terminal in its own window and uses the SSH user, port, key or password prompt configured in the profile. <code>ls</code> lists files, <code>cd</code> changes directories, <code>get</code> downloads and <code>put</code> uploads. SFTP and SCP use the same SSH protection; this session uses the macOS system client <code>/usr/bin/sftp</code>.</p>",
  "help.mosh.h": "Mosh terminal",
  "help.mosh.body": "<p>Mosh is a resilient terminal connection for changing or unreliable networks. Sign-in happens over SSH; the session then uses UDP, normally ports 60000–61000. The target needs <code>mosh-server</code>. RemoteDeskRDP does not bundle Mosh: it uses an existing local installation at <code>/opt/homebrew/bin/mosh</code>, <code>/usr/local/bin/mosh</code> or <code>/usr/bin/mosh</code>. This keeps the distributed app free from a Mosh licence obligation.</p>",
  "help.object.h": "S3 and OpenStack Swift",
  "help.object.body": "<p>S3 and Swift are object stores rather than drives: they contain buckets or containers with objects and optional prefixes. RemoteDeskRDP shows them in a dedicated window and supports navigation, upload, download and deletion of individual objects.</p><p>For S3, the endpoint, region and access key are stored in the profile; the secret access key stays exclusively in the macOS keychain. The default path style suits MinIO and many S3-compatible services. Swift uses Keystone v3; the user name and project stay in the profile and the password stays in the keychain. The app obtains a fresh token for every load.</p>",

  "help.gateway.h": "Connecting through a gateway",
  "help.gateway.body": "<p>An RD Gateway (also called RDS gateway server) accepts the connection from outside and passes it on to the actual machine inside the company network. It tunnels RDP over HTTPS, which is why the port is usually 443. Under GATEWAY enter only the name – port, protocol and slashes do not belong in that field, the port has one of its own.</p><p>If the gateway user, domain and password stay empty, the session credentials are reused. That is the normal case. Only fill in the three fields when the gateway requires its own account; the password then goes into the keychain as a separate entry.</p><p>Special characters in the gateway password are allowed. RemoteDeskRDP escapes comma, backslash and both quote characters by itself before handing the details to FreeRDP – without that escaping a quote character would make FreeRDP silently skip the gateway and try to reach the target directly.</p>",

  "help.udp.h": "UDP and video",
  "help.udp.body": "<p>RDP attempts UDP multitransport and falls back to TCP when UDP is unavailable. Video is not a guaranteed feature; where server, network and codecs allow it, it is passed through as well as possible.</p>",
  "help.reconnect.h": "Reconnecting after a dropout",
  "help.reconnect.body": "<p>Without this switch FreeRDP ends the session for good on the first lost beat – reconnection is off by default there. With it on, the client tries up to twenty times to resume the existing session; windows, running programs and the clipboard survive.</p><p>This only takes effect if the server issued a reconnection cookie at logon. If it did not, the switch stays without effect – it cannot do harm.</p><p>A common cause of dropouts are virtual machines the host suspends because they appear idle. In Parallels the setting is called “Pause idle” under Configure ▸ Options ▸ Optimization; while it is on, the machine freezes regularly during an RDP session.</p>",

  "help.display.h": "Window size and resolution",
  "help.display.body": "<p>The window mode sets the initial size: fixed resolution, usable screen area or full screen. “Follow with the resolution” changes the server resolution while enlarging and needs the server's display control channel. Without it, “Scale the image to the window” gives you a freely resizable window whose image grows along. In full screen ⌃⌥⏎ switches back.</p>",

  "help.files.h": "Exchanging files",
  "help.files.body": "<p>Copy and paste works in both directions, for text as well as for files. Files from the session are only downloaded to the Mac when you paste; for large files that takes accordingly long.</p><p><b>Dragging from the Mac into the session:</b> simply drag files out of the Finder into the session window and drop them there. They land where you release them.</p><p><b>Dragging from the session onto the Mac:</b> click the file in the guest system, then <b>hold the Option key ⌥</b> and drag out of the window with the mouse button held down. The Option key is necessary because RDP has no drag and drop: the client cannot see what is being dragged inside the guest system, and only treats the Option key as the instruction to copy the selection and continue the drag on the Mac. Without it, every move out of the window with the button held – while selecting text, for instance – would trigger the operation.</p>",

  "help.folders.h": "Folders instead of single files",
  "help.folders.body": "<p>Whether whole folders are transferred is decided by the guest system – not by RemoteDeskRDP.</p><ul><li><b>Windows:</b> folders arrive with their subfolders in both directions.</li><li><b>Linux with xrdp:</b> folders are rejected in both directions. Single files work, several at once as well.</li></ul><p>The limit is in xrdp itself: its <i>chansrv</i> service discards on receipt every entry that is a directory or that sits in a subfolder, and in the other direction it never offers folders in the first place. Because the entire list therefore arrives empty, the guest system reports an empty file name and “operation not supported”. A client cannot change this – RemoteDeskRDP sends a complete, protocol-conformant list, as the cross-check against Windows shows.</p><p>For folders on Linux targets use either the <b>folder share</b> – it appears in the guest as a drive and does not have this restriction – or pack the folder into an archive beforehand and transfer that.</p>",

  "help.limits.h": "Time limits when exchanging files",
  "help.limits.body": "<p>RDP does not transfer files in one go but piece by piece on request. Several deadlines follow from that; when one is exceeded, the operation in question aborts silently.</p><ul><li><b>Dropping into the session:</b> 3 seconds until the guest system confirms the offered file list. After that 150 milliseconds pass before the paste is triggered.</li><li><b>Dragging out of the session:</b> 10 seconds until the guest system reports, after the copy, which files it has.</li><li><b>Fetching file names:</b> 10 seconds.</li><li><b>Downloading contents:</b> 30 seconds per chunk, at most 10 minutes in total. There is no size limit.</li></ul><p>When dragging out, one further limit applies that does not come from a clock: macOS can only begin a drag while the mouse button is held down. If the download takes longer than you hold, no drag begins – but the files are then on the clipboard and can be pasted.</p>",

  "help.printer.h": "Sharing printers",
  "help.printer.body": "<p>This switch passes every printer set up on the Mac through to the session – RemoteDeskRDP hands <i>/printer</i> to FreeRDP, which reads the queues via CUPS. In the guest system they appear as printers of their own; anything printed from there comes out on the Mac.</p><p>Whether that succeeds is decided by the other end: it needs a driver for the reported device. Windows usually brings something suitable along. Without a driver the printer does show up but accepts no jobs. If the printer list on the Mac changes, that only takes effect in a newly established session.</p>",

  "help.smartcard.h": "Sharing smart cards",
  "help.smartcard.body": "<p>Passes attached card readers through to the session – for signing in by card, for digital signatures or for VPN access inside the guest system. macOS brings the necessary substructure (PCSC) along itself; no additional service needs to be set up.</p>",

  "help.video.h": "Accelerating video playback",
  "help.video.body": "<p>When the guest system plays video, it can transfer the picture stream separately as H.264 instead of sending it over the ordinary screen path. That saves bandwidth and makes moving images smoother. The server has to offer the channel; if it does not, the switch has no effect.</p><p><b>Not included: webcams.</b> Passing a camera from the Mac into the session is not possible with FreeRDP on macOS – the channel intended for it only has a Linux backend. This switch concerns exclusively video played <i>inside</i> the session.</p>",

  "help.security.h": "Security",
  "help.security.body": "<p>Credentials belong in the macOS keychain. Certificate changes are shown by default. Never ignore certificates blindly.</p><p>When a profile is deleted, RemoteDeskRDP also removes its passwords from the keychain – both the session one and the gateway one. Nothing orphaned is left behind.</p>",

  "help.appearance.h": "Appearance and language",
  "help.appearance.body": "<p>At the bottom of the sidebar there are two small switches. The left one cycles the appearance between <b>Auto</b>, <b>Light</b> and <b>Dark</b>, the right one the language between <b>Auto</b>, <b>German</b> and <b>English</b>. In the Auto position the app follows the macOS setting.</p><p>Both apply to the program as a whole rather than per connection, and they survive a restart. The session window itself is drawn by the guest system and is therefore not switched along.</p>",

  "help.dualbeam.h": "DualBeam",
  "help.dualbeam.body": "<p>RemoteDeskRDP is a standalone app and will accept versioned profile calls from DualBeam. That keeps the integration possible without a fixed plugin API.</p>",

  // ─── License ────────────────────────────────────────────────────
  "side.license": "§ License",
  "license.eyebrow": "LEGAL",
  "license.title": "License",
  "license.close": "Close",

  "license.mit.h": "RemoteDeskRDP – MIT License",
  "license.mit.body":
    "<p>RemoteDeskRDP is free and open-source software. Its source code may be used, copied, modified, merged, published, distributed, sublicensed and sold under the terms of the MIT License.</p>" +
    "<p><b>Copyright © 2026 Norbert Jander</b></p>" +
    "<p>Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:</p>" +
    "<p>The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.</p>" +
    "<p>THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.</p>",

  "license.modules.h": "Modules used and their licences",
  "license.modules.body":
    "<p>RemoteDeskRDP ships the following third-party components. The full licence text of each project is authoritative.</p>" +
    "<p><b>Apache-2.0</b></p>" +
    "<ul><li><b>FreeRDP 3.26.0</b> – RDP protocol (<code>libfreerdp3</code>, <code>libfreerdp-client3</code>, <code>libwinpr3</code>, <code>libwinpr-tools3</code>, <code>libfreerdp-server3</code>, <code>libfreerdp-server-proxy3</code>, <code>libMacFreeRDP-library</code>)</li>" +
    "<li><b>OpenSSL 3</b> – encryption (<code>libssl</code>, <code>libcrypto</code>)</li></ul>" +
    "<p><b>GNU LGPL 2.1 or later</b></p>" +
    "<ul><li><b>FFmpeg 8.0.1</b> – image scaling (<code>libavutil</code>, <code>libswscale</code>). Built without <code>--enable-gpl</code> and without <code>--enable-nonfree</code>, so the LGPL applies.</li>" +
    "<li><b>libusb 1.0.29</b> – USB redirection</li></ul>" +
    "<p><b>zlib licence</b></p>" +
    "<ul><li><b>SDL 3.2.28</b> – window, input, rendering (<code>libSDL3</code>), plus <code>libSDL3_image</code> and <code>libSDL3_ttf</code></li>" +
    "<li><b>zlib 1.4.1.1</b> – data compression</li></ul>" +
    "<p><b>BSD-2-Clause</b></p>" +
    "<ul><li><b>OpenH264</b> (Cisco Systems) – H.264 video coding</li></ul>" +
    "<p><b>BSD-3-Clause</b></p>" +
    "<ul><li><b>Opus</b> (Xiph.Org and others) – audio coding</li>" +
    "<li><b>uriparser</b> – URI parsing</li></ul>" +
    "<p><b>MIT</b></p>" +
    "<ul><li><b>json-c</b> – JSON parsing</li>" +
    "<li><b>xterm.js 5</b> and <b>portable-pty 0.9</b> – embedded SSH terminal and pseudo-terminal</li></ul>" +
    "<p><b>Fraunhofer FDK AAC Codec Library</b></p>" +
    "<ul><li><b>fdk-aac 2.0.3</b> – audio transport for a running session. Fraunhofer-Gesellschaft's own licence; it grants <b>no patent licence</b>. Shipping it in a product may require a separate patent licence; the licence points to Via Licensing or the patent holders directly. AAC serves the audio channel (<code>rdpsnd</code>) alone, not the creation or distribution of audio files: Windows peers negotiate about 160 kbit/s with it, whereas PCM without it would take about 1,400 kbit/s.</li></ul>" +
    "<p><b>Application framework</b></p>" +
    "<ul><li><b>Tauri 2</b> plus <code>tauri-plugin-dialog</code> and <code>tauri-plugin-deep-link</code> – MIT or Apache-2.0</li>" +
    "<li><b>serde</b>, <b>serde_json</b>, <b>dirs</b>, <b>security-framework</b>, <b>reqwest</b>, <b>chrono</b>, <b>hmac</b>, <b>sha2</b>, <b>hex</b> and <b>quick-xml</b> – MIT or Apache-2.0</li>" +
    "<li><b>SolidJS</b>, <b>Vite</b> – MIT</li>" +
    "<li><b>TypeScript</b> – Apache-2.0</li></ul>",

  "license.novnc.h": "noVNC – Mozilla Public License 2.0",
  "license.novnc.body":
    "<p>RemoteDeskRDP includes <b>noVNC 1.7.0</b> as its integrated browser-based standard RFB/VNC display. noVNC is licensed under the <b>Mozilla Public License 2.0 (MPL-2.0)</b>.</p>" +
    "<p>The full licence text is included with the app at <code>RemoteDeskRDP.app/Contents/Resources/resources/licenses/noVNC-LICENSE.txt</code>. Source code is available from <a href=\"https://github.com/novnc/noVNC\">github.com/novnc/noVNC</a>.</p>",

  "license.source.h": "Source code of third-party components",
  "license.source.body":
    "<p>The libraries under the <b>GNU LGPL 2.1</b> (FFmpeg, libusb) and <b>fdk-aac</b> carry an obligation to make the corresponding source code available. All of these libraries ship as separate, dynamically loaded files inside the application bundle and can be replaced there.</p>" +
    "<p>The source code of all third-party components is available unchanged from the respective projects; the versions used are named above. On request the author will supply the sources used free of charge.</p>" +
    "<p>The full licence texts ship with the program. You will find them inside the application bundle under <code>RemoteDeskRDP.app/Contents/Resources/resources/freerdp/</code> – FreeRDP's Apache-2.0 licence as <code>FREERDP-LICENSE.txt</code>, all others in the <code>licenses/</code> subfolder. That folder also contains the MPL-2.0 licence for noVNC and the MIT licence texts for xterm.js and portable-pty.</p>",

  "license.freerdp.h": "Modified files in FreeRDP and SDL",
  "license.freerdp.body":
    "<p>The Apache-2.0 licence requires modified files to be marked as such. RemoteDeskRDP modifies <b>13 files</b> in FreeRDP 3.26.0 and <b>3 files</b> in SDL 3.2.28:</p>" +
    "<p><b>FreeRDP 3.26.0</b></p>" +
    "<ul><li><code>scripts/bundle-mac-os.sh</code> – also builds the Cocoa client.</li>" +
    "<li><code>client/Mac/Keyboard.m</code> – fixes a build error that only appears when the Cocoa client is built.</li>" +
    "<li><code>client/Mac/cli/MainMenu.xib</code> – removes the fixed window size of 1024 × 768.</li>" +
    "<li><code>client/Mac/cli/AppDelegate.h</code> – declaration for <code>mac_set_view_size</code>.</li>" +
    "<li><code>client/Mac/cli/AppDelegate.m</code> – lets the session view grow with the window.</li>" +
    "<li><code>client/SDL/SDL3/sdl_freerdp.cpp</code> – presents once per frame instead of once per packet.</li>" +
    "<li><code>client/SDL/SDL3/sdl_context.cpp</code> and <code>.hpp</code> – separates drawing from presenting; accepts dropped files and detects dragging out.</li>" +
    "<li><code>client/SDL/SDL3/sdl_window.cpp</code> – disables alpha blending on the frame textures; fixes banding at 24-bit and a black screen at 32-bit colour depth.</li>" +
    "<li><code>client/SDL/SDL3/sdl_clip.cpp</code> and <code>.hpp</code> – reports the clipboard after connection setup, transfers files in both directions and handles drag and drop.</li>" +
    "<li><code>client/common/client_cliprdr_file.c</code> – fetches files from the session without FUSE and fixes a percent-encoding bug in file names.</li>" +
    "<li><code>include/freerdp/client/client_cliprdr_file.h</code> – adds <code>cliprdr_file_context_wait_for_files</code>.</li></ul>" +
    "<p><b>SDL 3.2.28</b></p>" +
    "<ul><li><code>src/video/cocoa/SDL_cocoaclipboard.m</code> – adds <code>text/uri-list</code> in both directions and resolves Finder file reference URLs.</li>" +
    "<li><code>src/video/cocoa/SDL_cocoaevents.m</code> – polls the clipboard at a throttled rate and hides an empty window from the main nib.</li>" +
    "<li><code>src/video/cocoa/SDL_cocoawindow.m</code> – makes it possible to start a macOS drag operation.</li></ul>" +
    "<p>Two of the changes fix bugs in FreeRDP itself: the one in <code>client_cliprdr_file.c</code> is reported there as <b>FreeRDP/FreeRDP#13130</b>, the one in <code>sdl_window.cpp</code> also affects the current 3.30.0 release. All other changes are adaptations for RemoteDeskRDP. The complete changes are included as patch files.</p>",

  // ─── Backend error messages ─────────────────────────────────────
  "err.profileNeedsIdAndName": "A profile needs an ID and a name.",
  "err.hostRequired": "Please enter a valid host name or IP address.",
  "err.portRange": "Ports must be between 1 and 65535.",
  "err.windowTooSmall": "The window size must be at least 640 × 480 pixels.",
  "err.oddDimensions": "Width and height must be even numbers.",
  "err.gatewayHostRequired": "Please enter a valid gateway host name.",
  "err.gatewayHostSpace": "The gateway host name must not contain spaces.",
  "err.gatewayHostColon": "The gateway host name must not contain a colon – the port belongs in its own field.",
  "err.gatewayPortRange": "The gateway port must be between 1 and 65535.",
  "err.shareNeedsNameAndPath": "A folder share needs a name and a path.",
  "err.shareNoComma": "The folder share “{0}” must not contain a comma – FreeRDP uses it to separate name and path.",
  "err.badProfileId": "Invalid profile ID for the keychain.",
  "err.emptyPassword": "The password must not be empty or contain line breaks.",
  "err.keychain": "Keychain: {0}",
  "err.keychainOnlyMacos": "The macOS keychain is only available on macOS.",
  "err.profileDirectory": "Profile folder: {0}",
  "err.profileFileUnreadable": "Could not read the profile file: {0}",
  "err.profileFileInvalid": "Invalid profile file: {0}",
  "err.profileWrite": "Could not write the profile file: {0}",
  "err.profileLock": "The profile file is currently locked.",
  "err.argumentLineBreak": "This field must not contain a line break.",
  "err.xquartz": "XQuartz could not be started: {0}",
  "err.freerdpStart": "FreeRDP could not be started: {0}",
  "err.freerdpInput": "FreeRDP input: {0}",
  "err.freerdpExited": "FreeRDP exited: {0}",
  "err.freerdpExitedUnexpectedly": "FreeRDP exited unexpectedly ({0}).",
  "err.rdpProfileRequired": "This setting is only available for RDP profiles.",
  "err.vncProfileRequired": "This function is only available for VNC profiles.",
  "err.vncAlreadyRunning": "A VNC session is already running for this profile.",
  "err.vncProxyStart": "Could not start the local VNC proxy: {0}",
  "err.vncEmbeddedOnly": "VNC sessions open inside RemoteDeskRDP.",
  "err.vncDisconnected": "The VNC connection was disconnected.",
  "err.sshProfileRequired": "This function is only available for SSH profiles.",
  "err.sshStart": "Could not start the SSH session: {0}",
  "err.sshNotRunning": "No SSH session is running for this profile.",
  "err.sshAlreadyRunning": "An SSH session is already running for this profile.",
  "err.sshWrite": "SSH input: {0}",
  "err.sshResize": "SSH terminal size: {0}",
  "err.sshStop": "Stop SSH session: {0}",
  "err.sshTerminalInvalid": "The SSH terminal type may only contain letters, numbers, dots, underscores, plus signs and hyphens.",
  "err.x11CommandInvalid": "The X11 launch command may contain at most 1024 characters.",
  "err.xquartzNotInstalled": "XQuartz is required for X11 forwarding. Install it first from xquartz.org.",
  "err.xquartzStart": "Could not start XQuartz: {0}",
  "err.xquartzNotReady": "XQuartz started but is not ready yet. Wait a moment and connect again.",
  "err.sshEmbeddedOnly": "SSH sessions open inside RemoteDeskRDP.",
  "err.terminalProfileRequired": "This function is only available for SSH, SFTP or Mosh profiles.",
  "err.sftpEmbeddedOnly": "SFTP sessions open inside RemoteDeskRDP.",
  "err.moshEmbeddedOnly": "Mosh sessions open inside RemoteDeskRDP.",
  "err.moshNotInstalled": "Mosh is not installed. For example, install it with: brew install mosh",
  "err.terminalClose": "Could not close terminal window: {0}",
  "err.objectProfileRequired": "This function is only available for S3 or Swift profiles.",
  "err.objectEmbeddedOnly": "S3 and Swift sessions open inside RemoteDeskRDP.",
  "err.objectEndpointInvalid": "Enter a valid HTTP or HTTPS endpoint URL.",
  "err.s3CredentialsRequired": "S3 needs a region and access key.",
  "err.swiftCredentialsRequired": "Swift needs a user name and Keystone project.",
  "err.objectSecretMissing": "The S3 secret or Swift password is missing from the macOS keychain.",
  "err.objectContainerInvalid": "The bucket or container name is invalid.",
  "err.objectKeyInvalid": "The object name is invalid.",
  "err.objectRequest": "Object storage request: {0}",
  "err.objectFileRead": "Could not read local file: {0}",
  "err.objectFileWrite": "Could not write local file: {0}",
  "err.objectDownloadExists": "A file with this name already exists at the destination. Choose another folder or rename it first.",
  "err.swiftTokenMissing": "Keystone did not return an access token.",
  "err.swiftStorageEndpointMissing": "Keystone did not provide a matching public object-storage endpoint.",
  "err.swiftPathInvalid": "The Keystone path must start with / and may not contain .. segments.",
  "err.nameAndHostRequired": "Please enter a name and a host.",
};
