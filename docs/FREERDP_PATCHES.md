# Änderungen an FreeRDP und SDL

RemoteDeskRDP liefert FreeRDP mit, baut es aber nicht unverändert. Zwei Patches
greifen ein, beide liegen in `scripts/patches/`:

| Patch | Ziel | Dateien |
|---|---|---|
| `freerdp-3.33.1-macos-client.patch` | FreeRDP, Tag `3.33.1` | 13 |
| `sdl-3.2.28-clipboard-poll.patch` | SDL, Tag `release-3.2.28` | 2 |

`scripts/build-freerdp-backend.sh` setzt die betroffenen Dateien zuerst per
`git checkout --` zurück und wendet dann den FreeRDP-Patch an — ein
halb zurückgenommener Baum kann so kein stillschweigend ungepatchtes Ergebnis
liefern. Den SDL-Patch reicht das Skript über die Umgebungsvariable
`REMOTEDESK_SDL_PATCH` an FreeRDPs `scripts/bundle-mac-os.sh` weiter, das ihn
nach dem Klonen anwendet.

Die Patchköpfe selbst gliedern nach Thema. Dieses Dokument gliedert nach
**Datei** und beantwortet je Datei: was wurde geändert und warum.

---

## Teil A — FreeRDP 3.33.1

### `scripts/bundle-mac-os.sh` (+11 Zeilen)

**Was:** Ergänzt `-DWITH_CLIENT_MAC=ON` in den CMake-Argumenten und wendet nach
dem Klonen der Abhängigkeiten den SDL-Patch aus `REMOTEDESK_SDL_PATCH` an.

**Warum:** Das Bundling-Skript baut den nativen Cocoa-Client nicht mit. Ohne
diese Zeile entsteht nur `sdl-freerdp`, und `MacFreeRDP` stünde nicht als
Rückfall zur Verfügung. Der SDL-Patch muss nach dem Klonen greifen, damit auch
ein frischer Quellbaum ihn erhält; ein vorhandener SDL-Checkout wird vorher
zurückgesetzt.

Seit 3.33 legt das Skript Quellen, Bau und Ergebnis unter `build-macos/` ab.

### `client/Mac/cli/MainMenu.xib` (3 Zeilen)

**Was:** Entfernt die Obergrenze der Fenstergröße und senkt die Untergrenze auf
640×480.

**Warum:** Das Fenster war auf 1024×768 festgenagelt — `minSize` war gleich
`maxSize`. Ein Vergrößern war schlicht unmöglich.

### `client/Mac/cli/AppDelegate.h` (+1 Zeile)

**Was:** Deklariert `mac_set_view_size(rdpContext*, MRDPView*)`.

**Warum:** Die Funktion wird jetzt aus `AppDelegate.m` heraus verwendet und
braucht eine sichtbare Deklaration.

### `client/Mac/cli/AppDelegate.m` (40 Zeilen)

**Was:** Hängt die Sitzungsansicht mit Autoresizing-Maske ein und hält in
`sessionViewDidResize:` `client_width`/`client_height` nach.

**Warum:** Die Ansicht wuchs nie mit dem Fenster mit. Zusätzlich ruft der
Kommandozeilen-Client `setScrollOffset:y:w:h:` nie auf, weshalb Skalierung und
Mauskoordinaten ohne diese Nachführung toter Code blieben.

### `client/Mac/CMakeLists.txt` (2 Zeilen)

**Was:** Übersetzt `client/common/oauth2.c` in der Cocoa-Bibliothek mit.

**Warum:** Fehler in FreeRDP 3.33 (auch auf `master`): Die Bibliothek
übersetzt `client/common/client.c` selbst, das `freerdp_oauth2_*` aufruft. Diese
Funktionen exportiert `libfreerdp-client3` nicht, also scheitert das Linken von
`MacFreeRDP` ohne diese Zeile.

### `client/SDL/SDL3/sdl_freerdp.cpp` (8 Zeilen)

**Was:** Die Bildschleife leert die Warteschlange erst vollständig
(`drawToWindows(rects, false)`) und stellt danach genau einmal dar
(`presentWindows()`).

**Warum:** Ohne die Änderung stellt der Client je eingegangenem
Aktualisierungspaket ein Bild dar. Jedes `SDL_RenderPresent` wartet auf den
Bildwechsel des Monitors — ein Bildaufbau aus zwanzig Paketen kostet zwanzig
Wartezyklen. Gemessen (mit 3.26.0) sank die Renderlast im Hauptthread von 47 auf
1 Prozent.

### `client/SDL/SDL3/sdl_context.cpp` (rund 250 Zeilen)

**Was:** Drei Bereiche.

1. `drawToWindow`/`drawToWindows` bekommen einen Schalter, ob dargestellt
   werden soll; `presentWindows()` kommt hinzu (Gegenstück zur Änderung in
   `sdl_freerdp.cpp`).
2. `handleEvent(SDL_DropEvent)` sammelt abgelegte Dateien, statt sie zu
   verwerfen, und übergibt sie beim Loslassen an den Zwischenablage-Pfad
   (`finishDrop`).
3. Die Maus-Ereignisse erkennen das Herausziehen aus dem Fenster
   (`startDragOut`): gedrückte linke Maustaste, gedrückte Wahltaste und ein
   Zeiger außerhalb des Fensters. Bei RemoteApp-Sitzungen bleibt das aus.

**Warum:** RDP hat keinen Kanal für Ziehen und Fallenlassen — die Kanalliste
kennt nur `cliprdr` und `rdpdr`. Der einzige Weg führt über die Zwischenablage.
Die Koordinatenumrechnung muss dieselbe Kette nehmen wie echte Mausereignisse
(`eventToPixelCoordinates`, `removeLocalScaling`, `applyMonitorOffset`), sonst
zeigt der Gastzeiger bei skalierten Fenstern woandershin. Die Wahltaste ist
bewusst verlangt, sonst löste jedes Herausfahren mit gedrückter Taste — etwa
beim Markieren von Text — den Vorgang aus.

### `client/SDL/SDL3/sdl_context.hpp` (25 Zeilen)

**Was:** Deklarationen zu den obigen Änderungen in `class SdlContext`.

**Warum:** Neue Methoden und Zustandsfelder für Darstellung und Ziehvorgang.

### `client/SDL/SDL3/sdl_window.cpp` (+15 Zeilen)

**Was:** Unter macOS vermessen `query()` und `rect()` die Monitore über
`queryDisplay` ohne Fenster; `createDummy` entfällt dort.

**Warum:** Zum Vermessen öffnet der SDL-Client je Display ein kleines Fenster,
schaltet es in den Vollbildmodus und liest Lage, Pixelgröße und Skalierung ab.
Unter macOS zieht dieses Fenster in einen eigenen Vollbild-Space — der ganze
Bildschirm wurde bei jedem Sitzungsstart kurz schwarz. Unter Wayland und X11
fragt FreeRDP bereits ohne Fenster ab.

### `client/SDL/SDL3/sdl_clip.cpp` (rund 190 Zeilen)

Zwei Richtungen für Ziehen und Fallenlassen. Das gewöhnliche Kopieren von
Dateien über die Zwischenablage erledigt FreeRDP seit 3.33 selbst
(`client_cliprdr_file_apple.m`).

**1. Mac → Sitzung.** `offerDroppedFiles()` kündigt dem Gast
`FileGroupDescriptorW` an, ohne die Mac-Zwischenablage anzutasten: Die Pfade
liegen prozentkodiert in `_drop_uri_list` und werden in
`ReceiveFormatDataRequestHandle` vor der Zwischenablage des Macs bedient.
`waitForDropAck()` wartet auf das Ereignis aus `ReceiveFormatListResponse`, denn
erst nach der Bestätigung liegen die Dateien in der Zwischenablage des Gasts —
ein sofortiges Strg+V ginge ins Leere.

**2. Sitzung → Mac.** `armServerFileWait()` und `fetchServerFiles()` warten nach
dem Strg+C im Gast auf dessen Formatliste, holen `FileGroupDescriptorW` und
laden die Dateien über `cliprdr_file_context_download_server_files()` sofort
herunter. Warten darf nur der SDL-Thread — ein wartender Kanal-Thread könnte die
Antworten des Servers nicht mehr annehmen.

### `client/SDL/SDL3/sdl_clip.hpp` (31 Zeilen)

**Was:** Deklarationen zu den obigen Punkten in `class sdlClip`.

**Warum:** Neue Methoden und die Felder `_drop_uri_list`, `_drop_ack` und
`_server_files`.

### `client/common/client_cliprdr_file.c` (+41 Zeilen)

**Was:** Neue Funktion `cliprdr_file_context_download_server_files()`.

**Warum:** macOS braucht zu Beginn eines Ziehvorgangs fertige Dateien. FreeRDP
holt Serverdateien seit 3.33 erst beim Einfügen ab. Die neue Funktion nutzt
dieselben Bausteine (`apple_set_server_files`, `apple_provide_file`), lädt aber
sofort alles herunter und liefert die Pfade zeilenweise — Windows-Dateinamen
können keinen Zeilenumbruch enthalten.

### `include/freerdp/client/client_cliprdr_file.h` (+17 Zeilen)

**Was:** Deklaration samt Dokumentationskommentar.

**Warum:** Der Kommentar hält fest, dass die Funktion blockiert und nur aus dem
SDL-Thread gerufen werden darf, nicht aus dem Kanal-Thread.

### Entfallen gegenüber 3.26.0

FreeRDP löst inzwischen selbst: den Baufehler in `client/Mac/Keyboard.m`, die
leere Formatliste beim Aushandeln der Zwischenablage, das Kopieren von Dateien
in beide Richtungen ohne FUSE (`WITH_CLIPRDR_FILE_APPLE`), den Fehler bei der
Prozentkodierung ([#13130](https://github.com/FreeRDP/FreeRDP/issues/13130)),
das Alpha-Blenden der Bildtexturen und den RAIL-Handshake.

---

## Teil B — SDL 3.2.28

FreeRDPs Bundling-Skript klont SDL selbst. Der Patch greift ausschließlich im
Cocoa-Treiber.

### `src/video/cocoa/SDL_cocoaevents.m` (33 Zeilen)

**Was:** Zwei Eingriffe.

1. `Cocoa_PumpEvents` gleicht die Zwischenablage zusätzlich gedrosselt ab
   (höchstens alle 250 ms) über `Cocoa_CheckClipboardUpdate`.
2. `LoadMainMenuNibIfAvailable` blendet Fenster aus dem geladenen Nib mit
   `orderOut:` aus.

**Warum:**

Zu 1: macOS meldet Änderungen der Zwischenablage nicht als Ereignis. SDL glich
sie deshalb nur in `windowDidBecomeKey` ab — solange das RDP-Fenster den
Tastaturfokus behielt, bemerkte SDL nie, dass in einem anderen Programm etwas
kopiert wurde. Der Client meldete dem Server dauerhaft
`ClientFormatList: numFormats: 0`, Einfügen in der Sitzung blieb leer. Der
Abgleich kostet nur einen Vergleich von `[NSPasteboard changeCount]` und belastet
den Renderpfad nicht.

Zu 2: Enthält das Bundle ein Main-Nib, lädt SDL es für das Menü. Cocoa zeigt
dabei jedes Fenster an, das im Nib als „Visible At Launch" markiert ist. Der
SDL-Client liegt im Bundle des Cocoa-Clients und erbte dessen `MainMenu.nib` mit
einem Sitzungsfenster von 1024×768 — bei jeder Sitzung stand ein zweites, leeres
Fenster mit dem Titel „FreeRDP" daneben. Nachgemessen mit
`CGWindowListCopyWindowInfo`: ein Prozess, zwei Fenster. Das Menü bleibt
erhalten; da SDL seine Fenster ausschließlich über `SDL_CreateWindow` anlegt,
kann ein Fenster aus dem Nib nie ein SDL-Fenster sein.

### `src/video/cocoa/SDL_cocoawindow.m` (+95 Zeilen)

**Was:** Neu hinzu kommen die Klasse `RemoteDeskDragSource` (implementiert
`NSDraggingSource`) und die exportierte Funktion `RemoteDesk_StartFileDrag()`.

**Warum:** SDL kann Ablegen nur empfangen (`performDragOperation`), aber selbst
keinen Ziehvorgang beginnen — eine öffentliche Schnittstelle dafür gibt es
nicht. Der FreeRDP-Client holt die Dateien aus dem Gast über die Zwischenablage
ab und übergibt die fertigen Pfade hier.

Zwei Feinheiten, die nicht offensichtlich sind:

- Das Ereignis wird **frisch erzeugt**, statt `currentEvent` zu benutzen —
  zwischen dem Herausziehen und dem Ende des Downloads können Sekunden liegen,
  das zuletzt verarbeitete Ereignis taugt dann nicht mehr.
- Die Quelle liegt in einer **statischen Variablen**
  (`s_remotedesk_drag_source`), weil der Ziehvorgang sie nur schwach hält; ohne
  diese Referenz würde sie vorzeitig freigegeben.

Der Vorgang meldet `NSDragOperationCopy` (Kopieren, nicht Verschieben), da die
Dateien in einem temporären Ordner liegen.

---

## Übersicht

Umfang laut `git diff --stat` in beiden Quellbäumen:

| Datei | Umfang | Grund in einem Satz |
|---|---|---|
| **FreeRDP** | **+618 / −15** | |
| `scripts/bundle-mac-os.sh` | +11 | Cocoa-Client mitbauen, SDL-Patch anwenden |
| `client/Mac/cli/MainMenu.xib` | 3 | Fenster war auf 1024×768 festgenagelt |
| `client/Mac/cli/AppDelegate.h` | +1 | Deklaration zu `mac_set_view_size` |
| `client/Mac/cli/AppDelegate.m` | 40 | Ansicht wuchs nicht mit dem Fenster mit |
| `client/Mac/CMakeLists.txt` | 2 | Cocoa-Client ließ sich nicht linken |
| `client/SDL/SDL3/sdl_freerdp.cpp` | 8 | Ein Present je Paket → Renderlast 47 % |
| `client/SDL/SDL3/sdl_context.cpp` | 248 | Darstellung entkoppeln; Ablegen und Herausziehen |
| `client/SDL/SDL3/sdl_context.hpp` | 25 | Deklarationen dazu |
| `client/SDL/SDL3/sdl_window.cpp` | +15 | Monitore ohne Vollbild-Probefenster vermessen |
| `client/SDL/SDL3/sdl_clip.cpp` | 191 | Ziehen und Fallenlassen in beide Richtungen |
| `client/SDL/SDL3/sdl_clip.hpp` | +31 | Deklarationen dazu |
| `client/common/client_cliprdr_file.c` | +41 | Serverdateien sofort herunterladen |
| `include/freerdp/client/client_cliprdr_file.h` | +17 | Deklaration dazu |
| **SDL** | **+127 / −1** | |
| `src/video/cocoa/SDL_cocoaevents.m` | 33 | Zwischenablage abgleichen; Geisterfenster ausblenden |
| `src/video/cocoa/SDL_cocoawindow.m` | +95 | Ziehvorgang starten (SDL kann das nicht) |

## Einordnung

Bis auf den Linkfehler in `client/Mac/CMakeLists.txt` sind alle verbliebenen
Änderungen **Anpassungen für RemoteDesk**: Funktionen, die FreeRDP und SDL auf
macOS nicht anbieten (Ziehen und Fallenlassen), oder Verhalten, das für diesen
Anwendungsfall nicht taugt (Fenstergröße, Renderpfad, Monitorvermessung,
Abgleich der Zwischenablage). Die früheren Fehlerbehebungen sind mit 3.33
upstream erledigt.

## Pflege

Die **Patchdateien sind die maßgebliche Quelle**, nicht die Arbeitskopien in
`.build/`. Jeder Bau setzt die betroffenen Dateien per `git checkout --` zurück
und wendet den Patch neu an — unaufbewahrte Änderungen im Quellbaum gehen dabei
verloren.

Patch nach einer Änderung neu erzeugen (Prosakopf erhalten, Diff neu):

```bash
cd .build/freerdp/FreeRDP
sed '/^diff --git/,$d' ../../../scripts/patches/freerdp-3.33.1-macos-client.patch > /tmp/head.txt
git diff -- $(git diff --name-only) > /tmp/diff.patch
cat /tmp/head.txt /tmp/diff.patch \
  > ../../../scripts/patches/freerdp-3.33.1-macos-client.patch
```

Vor dem Bau immer `git apply --check` laufen lassen — ein Patch, der nicht mehr
passt, würde sonst stillschweigend einen ungepatchten Bau erzeugen.
