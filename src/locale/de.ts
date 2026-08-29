// Deutsche Texte. Schlüssel müssen zu en.ts passen –
// der Test in i18n.test.ts hält das fest.

export const de: Record<string, string> = {
  // ─── Seitenleiste ───────────────────────────────────────────────
  "brand.subtitle": "RDP · VNC · SSH",
  "side.connections": "VERBINDUNGEN",
  "side.notConfigured": "nicht konfiguriert",
  "side.empty": "Noch keine gespeicherten Verbindungen.",
  "side.newProfile": "＋ Neue Verbindung",
  "side.help": "ⓘ Hilfe & Kompatibilität",
  "side.themeTitle": "Erscheinungsbild: {label}",
  "side.langTitle": "Sprache: {label}",
  "theme.auto": "Auto (folgt macOS)",
  "theme.light": "Hell",
  "theme.dark": "Dunkel",
  "lang.auto": "Auto (folgt macOS)",
  "lang.de": "Deutsch",
  "lang.en": "Englisch",

  // ─── Kopfzeile ──────────────────────────────────────────────────
  "top.eyebrow": "VERBINDUNGSPROFIL",
  "top.untitled": "Neue Verbindung",

  // ─── Zustandsmeldungen ──────────────────────────────────────────
  "state.ready": "Bereit für eine sichere Verbindung",
  "state.loadFailed": "Profile konnten nicht geladen werden: {error}",
  "state.profileLoaded": "Profil geladen",
  "state.newProfile": "Neue Verbindung anlegen",
  "state.passwordCleared": "Sitzungskennwort aus dem Schlüsselbund entfernt.",
  "state.passwordClearFailed": "Sitzungskennwort konnte nicht entfernt werden: {error}",
  "state.gatewayPasswordCleared": "Gateway-Kennwort aus dem Schlüsselbund entfernt.",
  "state.gatewayPasswordClearFailed": "Gateway-Kennwort konnte nicht entfernt werden: {error}",
  "state.saved": "Profil sicher gespeichert",
  "state.saveFailed": "Speichern fehlgeschlagen: {error}",
  "state.deleted": "Profil gelöscht",
  "state.deleteFailed": "Löschen fehlgeschlagen: {error}",
  "state.connecting": "Verbindung wird vorbereitet …",
  "state.linkUnknown": "Die angeforderte Verbindung ist hier nicht bekannt.",
  "state.connected": "{name} wurde in einem eigenen Sitzungsfenster gestartet.",
  "state.vncConnected": "{name} ist in dieser VNC-Sitzung verbunden.",
  "state.vncWindowOpened": "VNC-Sitzung für {name} wurde in einem eigenen Fenster geöffnet.",
  "state.vncClosed": "VNC-Sitzung wurde geschlossen.",
  "state.connectFailed": "Verbindung fehlgeschlagen: {error}",
  "state.folderTwice": "Ordner ist bereits freigegeben",
  "confirm.delete": "Dieses Verbindungsprofil wirklich löschen?",

  // ─── Identität ──────────────────────────────────────────────────
  "field.name": "Anzeigename",
  "field.namePlaceholder": "z. B. Büro-PC",
  "field.protocol": "Protokoll",
  "field.host": "Host oder IP-Adresse",
  "field.hostPlaceholder": "server.example.net oder 192.168.1.20",
  "field.username": "Benutzername",
  "field.domain": "Domäne",
  "field.optional": "optional",
  "field.password": "Passwort",
  "field.passwordPlaceholder": "Im macOS-Schlüsselbund speichern",
  "field.passwordNote": "Wird niemals im Profil oder als Prozessargument gespeichert.",
  "protocol.rdp": "RDP (Remotedesktop)",
  "protocol.vnc": "VNC (integrierte Sitzung)",
  "protocol.ssh": "SSH (Konsole)",
  "protocol.sftp": "SFTP/SCP (Dateikonsole)",
  "protocol.mosh": "Mosh (Konsole)",
  "protocol.s3": "S3 (Objekt-Speicher)",
  "protocol.swift": "OpenStack Swift (Objekt-Speicher)",
  "action.reveal": "Anzeigen",
  "action.hide": "Verbergen",
  "action.remove": "Entfernen",

  // ─── Objekt-Speicher ────────────────────────────────────────────
  "object.eyebrow": "OBJEKT-SPEICHER",
  "object.s3Title": "S3-Verbindung",
  "object.swiftTitle": "OpenStack-Swift-Verbindung",
  "object.keychain": "Geheimnis im Schlüsselbund",
  "object.s3Endpoint": "S3-Endpunkt-URL",
  "object.swiftEndpoint": "Swift-Standort / Endpunkt",
  "object.accessKey": "Access Key",
  "object.secretKey": "Secret Access Key",
  "object.swiftPassword": "Swift-Kennwort",
  "object.region": "Region",
  "object.container": "Start-Bucket / -Container",
  "object.containerOptional": "leer = alle Buckets / Container",
  "object.containerNote": "Optional. Im Sitzungsfenster kann jederzeit ein anderer Bucket oder Container gewählt werden.",
  "object.swiftProject": "Keystone-Projekt",
  "object.swiftTenant": "Keystone-Tenant",
  "object.swiftVersion": "Keystone-Version",
  "object.swiftVersionV3": "Keystone v3 — aktuell",
  "object.swiftVersionV2": "Keystone v2 — Altbestand",
  "object.swiftVersionNote": "v3 ist der heutige Standard. v2 ist nur für ältere Clouds vorgesehen.",
  "object.swiftPath": "Keystone-Pfad",
  "object.swiftPathNote": "Anbieterspezifischer Pfad, z. B. /identity/v3 bei Infomaniak.",
  "object.swiftUserDomain": "Benutzerdomäne",
  "object.swiftProjectDomain": "Projektdomäne",
  "object.secretPlaceholder": "Im macOS-Schlüsselbund speichern",
  "object.secretNote": "Wird niemals in profiles.json gespeichert oder an einen externen Prozess übergeben.",
  "object.pathStyle": "S3-Pfadstil verwenden (für MinIO und viele kompatible Anbieter)",
  "object.s3Note": "Unterstützt AWS S3 und S3-kompatible Anbieter. Die App signiert jede Anfrage lokal mit AWS Signature Version 4.",
  "object.swiftNote": "Unterstützt Keystone v3 sowie v2 für ältere Clouds und erneuert das Zugriffstoken bei jedem Ladevorgang. Der Keystone-Pfad ist frei konfigurierbar. Region kann leer bleiben, wenn nur ein Object-Store-Endpunkt vorliegt.",
  "object.close": "Objekt-Speicher schließen",
  "object.refresh": "Aktualisieren",
  "object.up": "Eine Ebene höher",
  "object.upload": "Hochladen",
  "object.uploadPick": "Datei zum Hochladen auswählen",
  "object.download": "Herunterladen",
  "object.downloadPick": "Zielordner auswählen",
  "object.uploaded": "{name} wurde hochgeladen.",
  "object.downloaded": "Gespeichert: {path}",
  "object.containers": "Buckets und Container",
  "object.folder": "Präfix",
  "object.empty": "Keine Einträge vorhanden.",
  "confirm.objectDelete": "„{name}“ wirklich aus dem Objekt-Speicher löschen?",

  // ─── Transport ──────────────────────────────────────────────────
  "transport.eyebrow": "TRANSPORT",
  "transport.title": "Port & Netzwerk",
  "transport.fallback": "↯ TCP-Fallback aktiv",
  "transport.udpPreferred": "UDP bevorzugen",
  "transport.autoReconnect": "Wiederverbinden",
  "transport.note": "UDP wird automatisch genutzt, wenn der Zielserver und die Firewall es erlauben. Direkte FreeRDP-Verbindungen verwenden dabei denselben Zielport wie RDP TCP; bei abweichenden Portwerten gilt der TCP-Port. Andernfalls bleibt die Sitzung über TCP aktiv.",
  "vnc.note": "Die VNC-Sitzung wird direkt in RemoteDeskRDP angezeigt. Das Kennwort wird beim Verbinden abgefragt und nicht im Profil gespeichert.",
  "vnc.disconnect": "VNC-Sitzung schließen",
  "vnc.passwordPrompt": "VNC-Kennwort für {name}:",
  "vnc.passwordRequired": "VNC-Kennwort erforderlich",
  "vnc.passwordHelp": "Kennwort für {name} eingeben.",
  "vnc.cancel": "Abbrechen",
  "vnc.savePassword": "Passwort im macOS-Schlüsselbund speichern",
  "ssh.terminal": "Terminaltyp",
  "ssh.terminalNote": "Wird als TERM an den SSH-Server übergeben. Standard: xterm-256color.",
  "ssh.terminal.xterm256": "xterm-256color — Standard",
  "ssh.terminal.xterm": "xterm — hohe Kompatibilität",
  "ssh.terminal.screen": "screen-256color — GNU-Screen-Modus",
  "ssh.terminal.tmux": "tmux-256color — tmux-Terminalmultiplexer",
  "ssh.terminal.vt100": "vt100 — sehr alte Systeme",
  "ssh.terminal.linux": "linux — lokale Linux-Konsole",
  "ssh.disconnect": "SSH-Sitzung schließen",
  "ssh.x11.enable": "X11-Weiterleitung aktivieren (-X)",
  "ssh.x11.command": "Linux-Anwendung automatisch starten",
  "ssh.x11.commandPlaceholder": "z. B. gedit oder xterm -e htop",
  "ssh.x11.commandNote": "Optionaler Befehl auf dem Linux-Rechner. Er wird nach erfolgreicher Anmeldung ausgeführt; beim Schließen der Anwendung endet die SSH-Sitzung.",
  "ssh.x11.closeXquartz": "XQuartz nach Ende der Anwendung schließen",
  "ssh.x11.note": "Gilt nur für grafische Linux-/Unix-Anwendungen, nicht für Windows-Programme. Benötigt XQuartz auf dem Mac. Falls es fehlt, zuerst von xquartz.org installieren; RemoteDeskRDP startet XQuartz beim Verbinden automatisch. Die optionale Beendigung schließt XQuartz erst, wenn keine weitere X11-Sitzung von RemoteDeskRDP aktiv ist; andere XQuartz-Anwendungen werden dabei ebenfalls beendet.",
  "terminal.port": "SSH-Port",
  "terminal.error": "Terminalfehler",
  "sftp.note": "Öffnet die integrierte SFTP-Dateikonsole über SSH. Mit get und put werden Dateien sicher herunter- bzw. hochgeladen; Schlüssel- und Kennwortabfragen erscheinen direkt im Fenster.",
  "mosh.note": "Mosh baut die Anmeldung über SSH auf und nutzt danach UDP (standardmäßig 60000–61000). Mosh muss lokal installiert sein; etwa mit Homebrew: brew install mosh.",
  "unit.port": "PORT",
  "unit.px": "PX",

  // ─── Gateway ────────────────────────────────────────────────────
  "gateway.eyebrow": "GATEWAY",
  "gateway.title": "RD-Gateway",
  "gateway.enable": "Über ein Gateway verbinden",
  "gateway.host": "Gateway-Host",
  "gateway.hostNote": "Ohne Port und ohne Protokoll – nur der Name.",
  "gateway.port": "Gateway-Port",
  "gateway.username": "Gateway-Benutzer",
  "gateway.domain": "Gateway-Domäne",
  "gateway.password": "Gateway-Passwort",
  "gateway.sameAsSession": "wie die Sitzung",
  "gateway.passwordNote": "Eigener Eintrag im macOS-Schlüsselbund, getrennt vom Sitzungskennwort.",
  "gateway.note": "Bleiben Benutzer, Domäne und Passwort leer, meldet sich RemoteDeskRDP am Gateway mit den Anmeldedaten der Sitzung an. Das ist der übliche Fall; eigene Angaben braucht es nur, wenn das Gateway ein anderes Konto verlangt.",

  // ─── Anzeige ────────────────────────────────────────────────────
  "display.eyebrow": "ANZEIGE",
  "display.title": "Fenster & Auflösung",
  "display.mode": "Fenstermodus",
  "display.mode.window": "Fenster mit fester Startauflösung",
  "display.mode.workarea": "Nutzbare Bildschirmfläche füllen",
  "display.mode.fullscreen": "Vollbild (⌃⌥⏎ wechselt)",
  "display.width": "Breite",
  "display.height": "Höhe",
  "display.resize": "Verhalten beim Vergrößern",
  "display.resize.dynamic": "Auflösung mitziehen (Server muss es unterstützen)",
  "display.resize.scale": "Bild auf Fenstergröße skalieren (immer möglich)",
  "display.resize.fixed": "Feste Auflösung, Fenster nicht änderbar",
  "display.depth": "Farbtiefe",
  "display.depth.auto": "Automatisch aushandeln",
  "display.depth.32": "32 Bit – True Color mit Alphakanal",
  "display.depth.24": "24 Bit – True Color",
  "display.depth.16": "16 Bit – High Color, spürbar weniger Daten",
  "display.depth.15": "15 Bit – High Color",
  "display.depth.8": "8 Bit – 256 Farben, für sehr langsame Leitungen",
  "display.depthNote": "Eine geringere Farbtiefe überträgt weniger Daten und hilft auf schmalen Leitungen. Im lokalen Netz ist „Automatisch“ die richtige Wahl.",
  "display.resizeNote": "„Auflösung mitziehen“ ändert beim Vergrößern die Serverauflösung und benötigt den Display-Control-Kanal. Bietet der Server ihn nicht an, liefert „Bild auf Fenstergröße skalieren“ ein frei veränderbares Fenster.",

  // ─── Sitzung ────────────────────────────────────────────────────
  "session.eyebrow": "SITZUNG",
  "session.title": "Arbeitsumgebung",
  "session.keychain": "⌘ Schlüsselbund",
  "session.clipboard": "Zwischenablage freigeben",
  "session.audio": "Audio weiterleiten",
  "session.printer": "Drucker freigeben",
  "session.smartcard": "Smartcard freigeben",
  "session.video": "Videowiedergabe beschleunigen",
  "session.shares": "Freigegebene Ordner",
  "session.addFolder": "Ordner hinzufügen",
  "session.pickFolder": "Ordner für die Sitzung freigeben",
  "session.shareFallbackName": "Freigabe",
  "session.noShares": "Kein Ordner freigegeben. Freigaben erscheinen im Gastsystem unter den umgeleiteten Laufwerken.",
  "session.shareName": "Freigabename",
  "session.removeShare": "Freigabe entfernen",
  "session.remove": "Entfernen",
  "session.certificates": "Zertifikate",
  "session.cert.prompt": "Unbekannte Zertifikate ablehnen",
  "session.cert.tofu": "Beim ersten Kontakt vertrauen",
  "session.cert.ignore": "Immer ignorieren (unsicher)",

  // ─── Fußleiste ──────────────────────────────────────────────────
  "action.deleteProfile": "Profil löschen",
  "action.saveProfile": "Profil sichern",
  "action.connect": "Verbinden",

  // ─── Hilfe ──────────────────────────────────────────────────────
  "help.eyebrow": "REMOTEDESKRDP",
  "help.title": "Hilfe & Kompatibilität",
  "help.close": "Schließen",

  "help.targets.h": "Unterstützte Ziele",
  "help.targets.body": "<p><b>Windows</b> über RDP oder VNC; <b>Linux</b> über RDP mit xrdp oder VNC; <b>macOS</b> über die Bildschirmfreigabe per VNC. RemoteDeskRDP zeigt VNC-Sitzungen direkt im App-Fenster an.</p>",

  "help.ports.h": "Ports",
  "help.ports.body": "<p>RDP TCP und UDP: 3389. Beide Werte sind pro Profil anpassbar.</p>",
  "help.vnc.h": "VNC-Sitzung",
  "help.vnc.body": "<p>VNC wird direkt im RemoteDeskRDP-Fenster angezeigt. Der integrierte Viewer spricht Standard-RFB und funktioniert damit mit gängigen VNC-Servern, einschließlich TightVNC, ohne serverspezifischen Pfad. Der Standardport ist 5900. Das Kennwort wird nur beim Verbinden abgefragt und nicht im Profil gespeichert. Für die lokale Verbindung zwischen Anzeige und App wird ausschließlich eine kurzlebige Loopback-Verbindung verwendet. RDP-Optionen wie Gateway, Laufwerke oder dynamische Auflösung gelten nur für RDP und bleiben bei VNC bewusst ausgeblendet.</p>",
  "help.ssh.h": "SSH-Konsole",
  "help.ssh.body": "<p>SSH öffnet eine interaktive Konsole in einem eigenen RemoteDeskRDP-Fenster. Die App nutzt den macOS-Systemclient <code>/usr/bin/ssh</code> mit einem Pseudo-Terminal; Passwort-, Schlüssel- und Hostschlüsselabfragen erscheinen daher direkt in der Konsole. Standardport ist 22. Benutzername, Port und Terminaltyp gehören ins Profil. Der Terminaltyp wird als <code>TERM</code> an den Server übergeben; <code>xterm-256color</code> ist die passende Vorgabe für die integrierte Konsole.</p><p>Optional schaltet das Profil X11-Weiterleitung mit <code>-X</code> ein. Das gilt nur für grafische Linux-/Unix-Anwendungen, nicht für Windows-Programme. Dafür muss <b>XQuartz</b> auf dem Mac installiert sein; RemoteDeskRDP startet es beim Verbinden und übergibt dessen Anzeige an SSH. Ein hinterlegter Linux-Befehl wie <code>gedit</code> oder <code>xterm -e htop</code> startet die Anwendung automatisch. Sie erscheint in einem XQuartz-Fenster auf dem Mac.</p>",
  "help.sftp.h": "SFTP/SCP-Dateikonsole",
  "help.sftp.body": "<p>SFTP öffnet eine sichere, interaktive Dateikonsole im eigenen Fenster und nutzt den im Profil hinterlegten SSH-Benutzer, Port sowie Schlüssel oder Kennwortabfrage. <code>ls</code> listet Dateien, <code>cd</code> wechselt Ordner, <code>get</code> lädt herunter und <code>put</code> lädt hoch. SFTP und SCP nutzen dieselbe SSH-Absicherung; die Sitzung verwendet den macOS-Systemclient <code>/usr/bin/sftp</code>.</p>",
  "help.mosh.h": "Mosh-Konsole",
  "help.mosh.body": "<p>Mosh ist eine robuste Terminalverbindung für wechselnde oder instabile Netze. Die Anmeldung läuft über SSH; anschließend fließt die Sitzung über UDP, standardmäßig im Bereich 60000–61000. Das Ziel benötigt <code>mosh-server</code>. RemoteDeskRDP bindet Mosh nicht mit ein, sondern verwendet eine vorhandene lokale Installation unter <code>/opt/homebrew/bin/mosh</code>, <code>/usr/local/bin/mosh</code> oder <code>/usr/bin/mosh</code>. So bleibt die App bei Weitergabe frei von einer Mosh-Lizenzbindung.</p>",
  "help.object.h": "S3 und OpenStack Swift",
  "help.object.body": "<p>S3 und Swift sind Objekt-Speicher, keine Laufwerke: Sie enthalten Buckets beziehungsweise Container mit Objekten und optionalen Präfixen. RemoteDeskRDP zeigt diese in einem eigenen Fenster an und unterstützt Navigieren, Hochladen, Herunterladen und Löschen einzelner Objekte.</p><p>Für S3 werden Endpunkt, Region und Access Key im Profil gespeichert; der Secret Access Key bleibt ausschließlich im macOS-Schlüsselbund. Der Standard-Pfadstil passt zu MinIO und vielen S3-kompatiblen Diensten. Swift verwendet Keystone v3; Benutzername und Projekt stehen im Profil, das Kennwort bleibt im Schlüsselbund. Die App holt für jeden Ladevorgang ein frisches Token.</p>",

  "help.gateway.h": "Über ein Gateway verbinden",
  "help.gateway.body": "<p>Ein RD-Gateway (auch RDS-Gatewayserver) nimmt die Verbindung von außen entgegen und reicht sie im Firmennetz an den eigentlichen Rechner weiter. Es tunnelt RDP dabei über HTTPS, weshalb der Port üblicherweise 443 lautet. Tragen Sie unter GATEWAY nur den Namen ein – Port, Protokoll und Schrägstriche gehören nicht in das Feld, der Port hat ein eigenes.</p><p>Bleiben Benutzer, Domäne und Passwort des Gateways leer, wird die Anmeldung der Sitzung mitbenutzt. Das ist der Normalfall. Nur wenn das Gateway ein eigenes Konto verlangt, füllen Sie die drei Felder; das Passwort landet dann als getrennter Eintrag im Schlüsselbund.</p><p>Sonderzeichen im Gateway-Passwort sind zulässig. RemoteDeskRDP maskiert Komma, Backslash und Anführungszeichen selbsttätig, bevor es die Angaben an FreeRDP übergibt – ohne diese Maskierung würde ein Anführungszeichen das Gateway stillschweigend übergehen und die Verbindung direkt zum Ziel versuchen.</p>",

  "help.udp.h": "UDP und Video",
  "help.udp.body": "<p>RDP versucht UDP-Multitransport und fällt bei nicht verfügbarem UDP auf TCP zurück. Video ist kein zugesagtes Feature; falls Server, Netzwerk und Codecs es ermöglichen, wird es bestmöglich weitergegeben.</p>",
  "help.reconnect.h": "Wiederverbinden nach einem Aussetzer",
  "help.reconnect.body": "<p>Ohne diesen Schalter beendet FreeRDP die Sitzung beim ersten verlorenen Takt endgültig – die Wiederverbindung ist dort standardmässig ausgeschaltet. Ist sie an, versucht der Client bis zu zwanzigmal, die bestehende Sitzung fortzusetzen; Fenster, angemeldete Programme und Zwischenablage bleiben erhalten.</p><p>Wirksam wird das nur, wenn der Server beim Anmelden eine Wiederverbindungs-Kennung ausgegeben hat. Tut er das nicht, bleibt der Schalter folgenlos – schaden kann er nicht.</p><p>Häufige Ursache für Aussetzer sind virtuelle Maschinen, die der Wirt anhält, weil sie untätig wirken. Bei Parallels heisst die Einstellung „Bei Untätigkeit anhalten“ unter Konfigurieren ▸ Optionen ▸ Optimierung; ist sie an, friert die Maschine während einer RDP-Sitzung regelmässig ein.</p>",

  "help.display.h": "Fenstergröße und Auflösung",
  "help.display.body": "<p>Der Fenstermodus legt die Startgröße fest: feste Auflösung, nutzbare Bildschirmfläche oder Vollbild. „Auflösung mitziehen“ ändert beim Vergrößern die Serverauflösung und benötigt den Display-Control-Kanal des Servers. Fehlt er, liefert „Bild auf Fenstergröße skalieren“ ein frei veränderbares Fenster, dessen Bild mitwächst. Im Vollbild wechselt ⌃⌥⏎ zurück.</p>",

  "help.files.h": "Dateien austauschen",
  "help.files.body": "<p>Kopieren und Einfügen funktioniert in beide Richtungen, für Text wie für Dateien. Dateien aus der Sitzung werden erst beim Einfügen auf dem Mac heruntergeladen; bei großen Dateien dauert das entsprechend.</p><p><b>Vom Mac in die Sitzung ziehen:</b> Dateien aus dem Finder einfach ins Sitzungsfenster ziehen und dort fallen lassen. Sie landen an der Stelle des Loslassens.</p><p><b>Aus der Sitzung auf den Mac ziehen:</b> Datei im Gastsystem anklicken, dann <b>die Wahltaste ⌥ gedrückt halten</b> und mit gedrückter Maustaste aus dem Fenster ziehen. Die Wahltaste ist nötig, weil RDP kein Ziehen und Fallenlassen kennt: Der Client kann nicht sehen, was im Gastsystem gezogen wird, und wertet erst die Wahltaste als Auftrag, die Auswahl zu kopieren und den Ziehvorgang auf dem Mac fortzusetzen. Ohne sie würde jedes Herausfahren mit gedrückter Maustaste – etwa beim Markieren von Text – den Vorgang auslösen.</p>",

  "help.folders.h": "Ordner statt einzelner Dateien",
  "help.folders.body": "<p>Ob ganze Ordner übertragen werden, entscheidet das Gastsystem – nicht RemoteDeskRDP.</p><ul><li><b>Windows:</b> Ordner kommen samt Unterordnern in beide Richtungen an.</li><li><b>Linux mit xrdp:</b> Ordner werden in beiden Richtungen abgelehnt. Einzelne Dateien funktionieren, auch mehrere auf einmal.</li></ul><p>Die Grenze liegt in xrdp selbst: Dessen Dienst <i>chansrv</i> verwirft beim Empfang jeden Eintrag, der ein Verzeichnis ist oder in einem Unterordner liegt, und bietet in der Gegenrichtung Ordner gar nicht erst an. Weil dadurch die gesamte Liste leer ankommt, meldet das Gastsystem einen leeren Dateinamen und „Vorgang wird nicht unterstützt“. Ein Client kann daran nichts ändern – RemoteDeskRDP sendet eine vollständige, protokollkonforme Liste, wie der Gegentest mit Windows zeigt.</p><p>Für Ordner auf Linux-Zielen daher entweder die <b>Ordnerfreigabe</b> nutzen – sie erscheint im Gast als Laufwerk und kennt diese Einschränkung nicht – oder den Ordner vorher in ein Archiv packen und dieses übertragen.</p>",

  "help.limits.h": "Zeitliche Grenzen beim Dateiaustausch",
  "help.limits.body": "<p>RDP überträgt Dateien nicht in einem Rutsch, sondern stückweise auf Anfrage. Daraus ergeben sich mehrere Fristen; wird eine überschritten, bricht der jeweilige Vorgang still ab.</p><ul><li><b>Ablegen in der Sitzung:</b> 3 Sekunden, bis das Gastsystem die angebotene Dateiliste bestätigt. Danach vergehen 150 Millisekunden, bevor das Einfügen ausgelöst wird.</li><li><b>Herausziehen aus der Sitzung:</b> 10 Sekunden, bis das Gastsystem nach dem Kopieren meldet, welche Dateien es hat.</li><li><b>Dateinamen abrufen:</b> 10 Sekunden.</li><li><b>Inhalte herunterladen:</b> 30 Sekunden je Teilstück, insgesamt höchstens 10 Minuten. Eine Größenbegrenzung gibt es nicht.</li></ul><p>Beim Herausziehen gilt zusätzlich eine Grenze, die nicht von einer Uhr kommt: macOS kann einen Ziehvorgang nur beginnen, solange die Maustaste gedrückt ist. Dauert das Herunterladen länger, als Sie halten, unterbleibt der Ziehvorgang – die Dateien liegen dann aber in der Zwischenablage und lassen sich einfügen.</p>",

  "help.printer.h": "Drucker freigeben",
  "help.printer.body": "<p>Der Schalter reicht alle auf dem Mac eingerichteten Drucker an die Sitzung weiter – RemoteDeskRDP übergibt dafür <i>/printer</i> an FreeRDP, das die Warteschlangen über CUPS ausliest. Im Gastsystem erscheinen sie als eigene Drucker; ein Ausdruck von dort landet auf dem Mac.</p><p>Ob das gelingt, entscheidet die Gegenstelle: Sie muss zum gemeldeten Gerät einen Treiber besitzen. Windows bringt dafür in der Regel etwas Passendes mit. Fehlt der Treiber, taucht der Drucker zwar auf, nimmt aber keine Aufträge an. Ändert sich die Druckerliste auf dem Mac, wirkt das erst in einer neu aufgebauten Sitzung.</p>",

  "help.smartcard.h": "Smartcard freigeben",
  "help.smartcard.body": "<p>Reicht angeschlossene Kartenleser an die Sitzung weiter – etwa zur Anmeldung per Karte, für digitale Signaturen oder VPN-Zugänge im Gastsystem. macOS bringt den nötigen Unterbau (PCSC) selbst mit; ein zusätzlicher Dienst ist nicht einzurichten.</p>",

  "help.video.h": "Videowiedergabe beschleunigen",
  "help.video.body": "<p>Spielt das Gastsystem Video ab, kann es den Bildstrom als H.264 gesondert übertragen, statt ihn über den gewöhnlichen Bildschirmweg zu schicken. Das spart Bandbreite und macht Bewegtbild flüssiger. Der Server muss den Kanal anbieten; tut er es nicht, bleibt der Schalter folgenlos.</p><p><b>Nicht enthalten: Webcams.</b> Eine Kamera vom Mac in die Sitzung durchzureichen ist mit FreeRDP unter macOS nicht möglich – der dafür vorgesehene Kanal besitzt nur ein Linux-Backend. Dieser Schalter betrifft ausschließlich Video, das <i>in</i> der Sitzung abgespielt wird.</p>",

  "help.security.h": "Sicherheit",
  "help.security.body": "<p>Zugangsdaten gehören in den macOS-Schlüsselbund. Zertifikatsänderungen werden standardmäßig angezeigt. Nie Zertifikate blind ignorieren.</p><p>Wird ein Profil gelöscht, entfernt RemoteDeskRDP auch dessen Kennwörter aus dem Schlüsselbund – sowohl das der Sitzung als auch das des Gateways. Es bleibt also nichts Verwaistes zurück.</p>",

  "help.appearance.h": "Erscheinungsbild und Sprache",
  "help.appearance.body": "<p>Unten in der Seitenleiste stehen zwei kleine Schalter. Der linke wechselt das Erscheinungsbild zwischen <b>Auto</b>, <b>Hell</b> und <b>Dunkel</b>, der rechte die Sprache zwischen <b>Auto</b>, <b>Deutsch</b> und <b>Englisch</b>. In der Stellung Auto folgt die App der Einstellung von macOS.</p><p>Beides gilt für das Programm als Ganzes, nicht je Verbindung, und bleibt über einen Neustart hinaus erhalten. Das Sitzungsfenster selbst wird vom Gastsystem gezeichnet und daher nicht mitgeschaltet.</p>",

  "help.dualbeam.h": "DualBeam",
  "help.dualbeam.body": "<p>RemoteDeskRDP ist eine Standalone-App und akzeptiert künftig versionierte Profil-Aufrufe von DualBeam. Damit bleibt die Integration ohne feste Plugin-API möglich.</p>",

  // ─── Lizenz ─────────────────────────────────────────────────────
  "side.license": "§ Lizenz",
  "license.eyebrow": "RECHTLICHES",
  "license.title": "Lizenz",
  "license.close": "Schließen",

  "license.mit.h": "RemoteDeskRDP – MIT-Lizenz",
  "license.mit.body":
    "<p>RemoteDeskRDP ist freie Open-Source-Software. Der Quellcode darf unter den Bedingungen der MIT-Lizenz verwendet, kopiert, verändert, zusammengeführt, veröffentlicht, verbreitet, unterlizenziert und verkauft werden.</p>" +
    "<p><b>Copyright © 2026 Norbert Jander</b></p>" +
    "<p>Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:</p>" +
    "<p>The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.</p>" +
    "<p>THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.</p>",

  "license.modules.h": "Verwendete Module und deren Lizenzen",
  "license.modules.body":
    "<p>RemoteDeskRDP liefert die folgenden Fremdkomponenten mit. Maßgeblich ist jeweils der vollständige Lizenztext des Projekts.</p>" +
    "<p><b>Apache-2.0</b></p>" +
    "<ul><li><b>FreeRDP 3.26.0</b> – RDP-Protokoll (<code>libfreerdp3</code>, <code>libfreerdp-client3</code>, <code>libwinpr3</code>, <code>libwinpr-tools3</code>, <code>libfreerdp-server3</code>, <code>libfreerdp-server-proxy3</code>, <code>libMacFreeRDP-library</code>)</li>" +
    "<li><b>OpenSSL 3</b> – Verschlüsselung (<code>libssl</code>, <code>libcrypto</code>)</li></ul>" +
    "<p><b>GNU LGPL 2.1 oder später</b></p>" +
    "<ul><li><b>FFmpeg 8.0.1</b> – Bildskalierung (<code>libavutil</code>, <code>libswscale</code>). Gebaut ohne <code>--enable-gpl</code> und ohne <code>--enable-nonfree</code>, es gilt daher die LGPL.</li>" +
    "<li><b>libusb 1.0.29</b> – USB-Weiterleitung</li></ul>" +
    "<p><b>zlib-Lizenz</b></p>" +
    "<ul><li><b>SDL 3.2.28</b> – Fenster, Eingabe, Darstellung (<code>libSDL3</code>), dazu <code>libSDL3_image</code> und <code>libSDL3_ttf</code></li>" +
    "<li><b>zlib 1.4.1.1</b> – Datenkomprimierung</li></ul>" +
    "<p><b>BSD-2-Clause</b></p>" +
    "<ul><li><b>OpenH264</b> (Cisco Systems) – H.264-Videokodierung</li></ul>" +
    "<p><b>BSD-3-Clause</b></p>" +
    "<ul><li><b>Opus</b> (Xiph.Org u. a.) – Audiokodierung</li>" +
    "<li><b>uriparser</b> – URI-Auswertung</li></ul>" +
    "<p><b>MIT</b></p>" +
    "<ul><li><b>json-c</b> – JSON-Auswertung</li>" +
    "<li><b>xterm.js 5</b> und <b>portable-pty 0.9</b> – eingebettete SSH-Konsole und Pseudo-Terminal</li></ul>" +
    "<p><b>Fraunhofer FDK AAC Codec Library</b></p>" +
    "<ul><li><b>fdk-aac 2.0.3</b> – Tonübertragung einer laufenden Sitzung. Eigene Lizenz der Fraunhofer-Gesellschaft; sie erteilt <b>keine Patentlizenz</b>. Für den Einsatz in einem Produkt kann eine gesonderte Patentlizenz erforderlich sein; die Lizenz nennt dafür Via Licensing oder die Patentinhaber unmittelbar. Verwendet wird AAC allein für den Tonkanal (<code>rdpsnd</code>), nicht zum Erzeugen oder Weitergeben von Audiodateien: Windows-Gegenstellen handeln damit rund 160 kbit/s aus, ohne AAC wären es über PCM rund 1 400 kbit/s.</li></ul>" +
    "<p><b>Programmgerüst der Anwendung</b></p>" +
    "<ul><li><b>Tauri 2</b> sowie <code>tauri-plugin-dialog</code> und <code>tauri-plugin-deep-link</code> – MIT oder Apache-2.0</li>" +
    "<li><b>serde</b>, <b>serde_json</b>, <b>dirs</b>, <b>security-framework</b>, <b>reqwest</b>, <b>chrono</b>, <b>hmac</b>, <b>sha2</b>, <b>hex</b> und <b>quick-xml</b> – MIT oder Apache-2.0</li>" +
    "<li><b>SolidJS</b>, <b>Vite</b> – MIT</li>" +
    "<li><b>TypeScript</b> – Apache-2.0</li></ul>",

  "license.novnc.h": "noVNC – Lizenzhinweis (MPL-2.0)",
  "license.novnc.body":
    "<p>RemoteDeskRDP enthält <b>noVNC 1.7.0</b> als integrierte, browserbasierte Standard-RFB/VNC-Anzeige. noVNC steht unter der <b>Mozilla Public License 2.0 (MPL-2.0)</b>.</p>" +
    "<p>Der vollständige Lizenztext liegt dem Programm bei unter <code>RemoteDeskRDP.app/Contents/Resources/resources/licenses/noVNC-LICENSE.txt</code>. Der Quelltext ist beim Projekt <a href=\"https://github.com/novnc/noVNC\">github.com/novnc/noVNC</a> verfügbar.</p>",

  "license.source.h": "Quelltext der Fremdkomponenten",
  "license.source.body":
    "<p>Für die unter der <b>GNU LGPL 2.1</b> stehenden Bibliotheken (FFmpeg, libusb) sowie für <b>fdk-aac</b> besteht die Pflicht, den zugehörigen Quelltext bereitzustellen. Alle genannten Bibliotheken sind als eigenständige, dynamisch geladene Dateien im Programmpaket abgelegt und können dort ausgetauscht werden.</p>" +
    "<p>Der Quelltext aller Fremdkomponenten ist unverändert bei den jeweiligen Projekten erhältlich; die verwendeten Fassungen sind oben mit Versionsnummer benannt. Auf Anfrage stellt der Urheber die verwendeten Quellen kostenfrei bereit.</p>" +
    "<p>Die vollständigen Lizenztexte liegen dem Programm bei. Sie finden sie im Programmpaket unter <code>RemoteDeskRDP.app/Contents/Resources/resources/freerdp/</code> – die Apache-2.0-Lizenz von FreeRDP als <code>FREERDP-LICENSE.txt</code>, alle übrigen im Unterordner <code>licenses/</code>. Dort liegen auch die MPL-2.0-Lizenz von noVNC sowie die MIT-Lizenztexte von xterm.js und portable-pty.</p>",

  "license.freerdp.h": "Geänderte Dateien in FreeRDP und SDL",
  "license.freerdp.body":
    "<p>Die Apache-2.0-Lizenz verlangt, geänderte Dateien kenntlich zu machen. RemoteDeskRDP verändert FreeRDP 3.26.0 in <b>13 Dateien</b> und SDL 3.2.28 in <b>3 Dateien</b>:</p>" +
    "<p><b>FreeRDP 3.26.0</b></p>" +
    "<ul><li><code>scripts/bundle-mac-os.sh</code> – baut zusätzlich den Cocoa-Client mit.</li>" +
    "<li><code>client/Mac/Keyboard.m</code> – behebt einen Baufehler, der erst beim Bauen des Cocoa-Clients auftritt.</li>" +
    "<li><code>client/Mac/cli/MainMenu.xib</code> – hebt die feste Fenstergröße von 1024 × 768 auf.</li>" +
    "<li><code>client/Mac/cli/AppDelegate.h</code> – Deklaration zu <code>mac_set_view_size</code>.</li>" +
    "<li><code>client/Mac/cli/AppDelegate.m</code> – lässt die Sitzungsansicht mit dem Fenster mitwachsen.</li>" +
    "<li><code>client/SDL/SDL3/sdl_freerdp.cpp</code> – stellt einmal je Bildaufbau dar statt einmal je Datenpaket.</li>" +
    "<li><code>client/SDL/SDL3/sdl_context.cpp</code> und <code>.hpp</code> – trennt Zeichnen und Darstellen; nimmt abgelegte Dateien entgegen und erkennt das Herausziehen.</li>" +
    "<li><code>client/SDL/SDL3/sdl_window.cpp</code> – schaltet das Alpha-Blenden der Bildtexturen ab; behebt Streifen bei 24 Bit und ein schwarzes Bild bei 32 Bit Farbtiefe.</li>" +
    "<li><code>client/SDL/SDL3/sdl_clip.cpp</code> und <code>.hpp</code> – meldet die Zwischenablage beim Verbindungsaufbau nach, überträgt Dateien in beide Richtungen und bedient Ziehen und Fallenlassen.</li>" +
    "<li><code>client/common/client_cliprdr_file.c</code> – holt Dateien aus der Sitzung ohne FUSE ab und behebt einen Fehler bei der Prozentkodierung von Datei­namen.</li>" +
    "<li><code>include/freerdp/client/client_cliprdr_file.h</code> – ergänzt <code>cliprdr_file_context_wait_for_files</code>.</li></ul>" +
    "<p><b>SDL 3.2.28</b></p>" +
    "<ul><li><code>src/video/cocoa/SDL_cocoaclipboard.m</code> – ergänzt <code>text/uri-list</code> in beide Richtungen und löst File-Reference-URLs des Finders auf.</li>" +
    "<li><code>src/video/cocoa/SDL_cocoaevents.m</code> – gleicht die Zwischenablage gedrosselt ab und blendet ein leeres Fenster aus dem Main-Nib aus.</li>" +
    "<li><code>src/video/cocoa/SDL_cocoawindow.m</code> – erlaubt es, einen macOS-Ziehvorgang zu starten.</li></ul>" +
    "<p>Zwei der Änderungen beheben Fehler in FreeRDP selbst: der in <code>client_cliprdr_file.c</code> ist dort unter <b>FreeRDP/FreeRDP#13130</b> gemeldet, der in <code>sdl_window.cpp</code> betrifft auch die neueste Fassung 3.30.0. Alle übrigen Änderungen sind Anpassungen für RemoteDeskRDP. Die vollständigen Änderungen liegen als Patchdateien bei.</p>",

  // ─── Fehlermeldungen des Backends ───────────────────────────────
  "err.profileNeedsIdAndName": "Ein Profil benötigt eine ID und einen Namen.",
  "err.hostRequired": "Bitte einen gültigen Hostnamen oder eine IP-Adresse angeben.",
  "err.portRange": "Ports müssen zwischen 1 und 65535 liegen.",
  "err.windowTooSmall": "Die Fenstergröße muss mindestens 640 × 480 Pixel betragen.",
  "err.oddDimensions": "Breite und Höhe müssen gerade Werte sein.",
  "err.gatewayHostRequired": "Bitte einen gültigen Gateway-Hostnamen angeben.",
  "err.gatewayHostSpace": "Der Gateway-Hostname darf keine Leerzeichen enthalten.",
  "err.gatewayHostColon": "Der Gateway-Hostname darf keinen Doppelpunkt enthalten – der Port gehört in das eigene Feld.",
  "err.gatewayPortRange": "Der Gateway-Port muss zwischen 1 und 65535 liegen.",
  "err.shareNeedsNameAndPath": "Eine Ordnerfreigabe braucht Name und Pfad.",
  "err.shareNoComma": "Die Ordnerfreigabe „{0}“ darf kein Komma enthalten – FreeRDP trennt damit Name und Pfad.",
  "err.badProfileId": "Ungültige Profil-ID für den Schlüsselbund.",
  "err.emptyPassword": "Das Kennwort darf nicht leer sein oder Zeilenumbrüche enthalten.",
  "err.keychain": "Schlüsselbund: {0}",
  "err.keychainOnlyMacos": "Der macOS-Schlüsselbund ist nur unter macOS verfügbar.",
  "err.profileDirectory": "Profilordner: {0}",
  "err.profileFileUnreadable": "Profildatei konnte nicht gelesen werden: {0}",
  "err.profileFileInvalid": "Ungültige Profildatei: {0}",
  "err.profileWrite": "Profildatei konnte nicht geschrieben werden: {0}",
  "err.profileLock": "Profildatei ist gerade gesperrt.",
  "err.argumentLineBreak": "Dieses Feld darf keinen Zeilenumbruch enthalten.",
  "err.xquartz": "XQuartz konnte nicht gestartet werden: {0}",
  "err.freerdpStart": "FreeRDP konnte nicht gestartet werden: {0}",
  "err.freerdpInput": "FreeRDP-Eingabe: {0}",
  "err.freerdpExited": "FreeRDP wurde beendet: {0}",
  "err.freerdpExitedUnexpectedly": "FreeRDP wurde unerwartet beendet ({0}).",
  "err.rdpProfileRequired": "Diese Einstellung ist nur für RDP-Profile verfügbar.",
  "err.vncProfileRequired": "Diese Funktion ist nur für VNC-Profile verfügbar.",
  "err.vncAlreadyRunning": "Für dieses Profil läuft bereits eine VNC-Sitzung.",
  "err.vncProxyStart": "Der lokale VNC-Proxy konnte nicht gestartet werden: {0}",
  "err.vncEmbeddedOnly": "VNC-Sitzungen werden in RemoteDeskRDP geöffnet.",
  "err.vncDisconnected": "Die VNC-Verbindung wurde getrennt.",
  "err.sshProfileRequired": "Diese Funktion ist nur für SSH-Profile verfügbar.",
  "err.sshStart": "Die SSH-Sitzung konnte nicht gestartet werden: {0}",
  "err.sshNotRunning": "Für dieses Profil läuft keine SSH-Sitzung.",
  "err.sshAlreadyRunning": "Für dieses Profil läuft bereits eine SSH-Sitzung.",
  "err.sshWrite": "SSH-Eingabe: {0}",
  "err.sshResize": "SSH-Terminalgröße: {0}",
  "err.sshStop": "SSH-Sitzung beenden: {0}",
  "err.sshTerminalInvalid": "Der SSH-Terminaltyp darf nur Buchstaben, Ziffern, Punkt, Unterstrich, Plus und Bindestrich enthalten.",
  "err.x11CommandInvalid": "Der X11-Startbefehl darf höchstens 1024 Zeichen lang sein.",
  "err.xquartzNotInstalled": "XQuartz ist für X11-Weiterleitung erforderlich. Bitte zuerst von xquartz.org installieren.",
  "err.xquartzStart": "XQuartz konnte nicht gestartet werden: {0}",
  "err.xquartzNotReady": "XQuartz wurde gestartet, war aber noch nicht bereit. Bitte kurz warten und erneut verbinden.",
  "err.sshEmbeddedOnly": "SSH-Sitzungen werden in RemoteDeskRDP geöffnet.",
  "err.terminalProfileRequired": "Diese Funktion ist nur für SSH-, SFTP- oder Mosh-Profile verfügbar.",
  "err.sftpEmbeddedOnly": "SFTP-Sitzungen werden in RemoteDeskRDP geöffnet.",
  "err.moshEmbeddedOnly": "Mosh-Sitzungen werden in RemoteDeskRDP geöffnet.",
  "err.moshNotInstalled": "Mosh ist nicht installiert. Installieren Sie es beispielsweise mit: brew install mosh",
  "err.terminalClose": "Terminalfenster konnte nicht geschlossen werden: {0}",
  "err.objectProfileRequired": "Diese Funktion ist nur für S3- oder Swift-Profile verfügbar.",
  "err.objectEmbeddedOnly": "S3- und Swift-Sitzungen werden in RemoteDeskRDP geöffnet.",
  "err.objectEndpointInvalid": "Bitte eine gültige HTTP- oder HTTPS-Endpunkt-URL angeben.",
  "err.s3CredentialsRequired": "S3 benötigt Region und Access Key.",
  "err.swiftCredentialsRequired": "Swift benötigt Benutzername und Keystone-Projekt.",
  "err.objectSecretMissing": "Das S3-Geheimnis bzw. Swift-Kennwort fehlt im macOS-Schlüsselbund.",
  "err.objectContainerInvalid": "Der Bucket- oder Containername ist ungültig.",
  "err.objectKeyInvalid": "Der Objektname ist ungültig.",
  "err.objectRequest": "Objekt-Speicher-Anfrage: {0}",
  "err.objectFileRead": "Lokale Datei konnte nicht gelesen werden: {0}",
  "err.objectFileWrite": "Lokale Datei konnte nicht geschrieben werden: {0}",
  "err.objectDownloadExists": "Am Ziel existiert bereits eine Datei mit diesem Namen. Bitte einen anderen Zielordner wählen oder die Datei zuerst umbenennen.",
  "err.swiftTokenMissing": "Keystone hat kein Zugriffstoken zurückgegeben.",
  "err.swiftStorageEndpointMissing": "Keystone liefert keinen passenden öffentlichen Object-Storage-Endpunkt.",
  "err.swiftPathInvalid": "Der Keystone-Pfad muss mit / beginnen und darf keine ..-Segmente enthalten.",
  "err.nameAndHostRequired": "Bitte Name und Host angeben.",
};
