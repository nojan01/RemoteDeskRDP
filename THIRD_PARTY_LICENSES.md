# Mitgelieferte Fremdkomponenten und ihre Lizenzen

RemoteDeskRDP liefert 21 Bibliotheken mit. Maßgeblich ist jeweils der
vollständige Lizenztext des Projekts. Alle Texte liegen dem Programm bei, im
Programmpaket unter
`RemoteDeskRDP.app/Contents/Resources/resources/freerdp/` — die Apache-2.0-Lizenz
von FreeRDP als `FREERDP-LICENSE.txt`, die übrigen dreizehn im Unterordner
`licenses/`.

## Übersicht

| Komponente | Version | Lizenz | Zweck |
|---|---|---|---|
| FreeRDP | 3.26.0 | Apache-2.0 | RDP-Protokoll |
| OpenSSL | 3 | Apache-2.0 | Verschlüsselung |
| FFmpeg | 8.0.1 | **LGPL-2.1-or-later** | Bildskalierung |
| libusb | 1.0.29 | **LGPL-2.1-or-later** | USB-Weiterleitung |
| SDL | 3.2.28 | zlib | Fenster, Eingabe, Darstellung |
| SDL_image, SDL_ttf | 3 | zlib | Bilder, Schriften |
| zlib | 1.4.1.1 | zlib | Datenkomprimierung |
| OpenH264 | master | BSD-2-Clause | H.264-Videokodierung |
| Opus | — | BSD-3-Clause | Audiokodierung |
| uriparser | — | BSD-3-Clause | URI-Auswertung |
| json-c | — | MIT | JSON-Auswertung |
| fdk-aac | 2.0.3 | **Fraunhofer FDK AAC** | AAC-Audiokodierung |

Die aus FreeRDP entstehenden Einzelbibliotheken sind `libfreerdp3`,
`libfreerdp-client3`, `libwinpr3`, `libwinpr-tools3`, `libfreerdp-server3`,
`libfreerdp-server-proxy3` und `libMacFreeRDP-library`.

## Programmgerüst der Anwendung

| Komponente | Lizenz |
|---|---|
| Tauri 2, `tauri-plugin-dialog`, `tauri-plugin-deep-link` | MIT oder Apache-2.0 |
| `serde`, `serde_json`, `dirs`, `security-framework` | MIT oder Apache-2.0 |
| `reqwest`, `chrono`, `hmac`, `sha2`, `hex`, `quick-xml` | MIT oder Apache-2.0 |
| SolidJS, Vite | MIT |
| TypeScript | Apache-2.0 |

## Punkte, die Beachtung verlangen

### FFmpeg steht unter LGPL, nicht unter GPL

`scripts/bundle-mac-os.sh` konfiguriert FFmpeg ohne `--enable-gpl` und ohne
`--enable-nonfree`:

```
--disable-all --enable-shared --disable-static --enable-swscale --disable-asm
--disable-libxcb --disable-securetransport --disable-xlib --enable-cross-compile
```

Damit gilt die LGPL-2.1-or-later. Würde eine dieser Schalterstellungen später
geändert, könnte FFmpeg unter die GPL fallen; dann müsste die Lizenzsituation
der Gesamtverteilung neu bewertet werden. **Vor jeder Änderung an dieser Zeile
prüfen.**

### LGPL verlangt Austauschbarkeit

FFmpeg und libusb liegen als eigenständige, dynamisch geladene `.dylib`-Dateien
im Programmpaket und lassen sich dort ersetzen. Damit bleibt die von der LGPL
verlangte Austauschbarkeit gewahrt. **Ein statisches Einbinden dieser beiden
Bibliotheken wäre nicht zulässig**, ohne zusätzlich Objektdateien zum Neubinden
bereitzustellen.

### fdk-aac erteilt keine Patentlizenz

Die Fraunhofer-Lizenz erlaubt Weitergabe in Quell- und Binärform ohne Gebühren,
sagt aber ausdrücklich:

> NO PATENT LICENSE

AAC ist patentbelastet. Für den Vertrieb eines Produkts kann daher eine
gesonderte Patentlizenz erforderlich sein — unabhängig von der
Urheberrechtslizenz. Die Lizenz nennt dafür Via Licensing oder die
Patentinhaber unmittelbar. Ebenfalls gefordert: Der vollständige Quelltext von
fdk-aac muss Empfängern der Binärfassung kostenfrei zugänglich sein.

**Wozu RemoteDeskRDP AAC verwendet.** Ausschliesslich für den Tonkanal einer
laufenden Sitzung (`rdpsnd`), nicht zum Erzeugen oder Weitergeben von
Audiodateien. Gemessen an zwei Gegenstellen:

| Gegenstelle | ausgehandeltes Format | Datenrate |
|---|---|---|
| Windows 11 | `WAVE_FORMAT_AAC_MS` | rund 160 kbit/s |
| Linux (xrdp) | `WAVE_FORMAT_PCM` | rund 1 400 kbit/s |

**Die Abhängigkeit ist verzichtbar.** In FreeRDP ist `WITH_FDK_AAC` von Haus aus
abgeschaltet; erst `scripts/bundle-mac-os.sh` setzt sie auf `ON`. Ohne fdk-aac
bleiben PCM und DVI-ADPCM übrig (`libfreerdp/codec/dsp.c`), der Ton läuft also
weiter — er belegt gegenüber AAC nur rund das Neunfache an Bandbreite. Wer die
Patentfrage vermeiden will, baut das Backend ohne diese Option.

### Pflicht zur Quelltextbereitstellung

Für FFmpeg, libusb (LGPL) und fdk-aac besteht die Pflicht, den zugehörigen
Quelltext bereitzustellen. Die verwendeten Fassungen sind oben mit
Versionsnummer benannt und unverändert bei den jeweiligen Projekten erhältlich.

### Apache-2.0 verlangt, geänderte Dateien zu kennzeichnen

Ziffer 4(b) der Apache-2.0-Lizenz verlangt, dass geänderte Dateien einen
deutlichen Hinweis tragen. RemoteDeskRDP verändert FreeRDP in 12 Dateien und SDL
in 3 Dateien; sie sind einzeln in
[docs/FREERDP_PATCHES.md](docs/FREERDP_PATCHES.md) und im Lizenzfenster
der Anwendung aufgeführt.
