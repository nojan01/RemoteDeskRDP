# TODO

## RDP-Optionen, die das Profil noch nicht hat

- [ ] **Mikrofon** (`/microphone`): Es gibt bisher nur Tonausgabe.
- [ ] **Mehrere Monitore** (`/multimon`, `/monitors`).
- [ ] **HiDPI-Skalierung** (`/scale`, `/scale-desktop`): wichtig auf Retina-Displays.
- [ ] **Admin-/Konsolensitzung** (`/admin`).
- [ ] **Tastaturlayout** (`/kbd`) und **Zeitzone**.
- [ ] **RemoteApp** (`/app`): einzelne Programme statt des ganzen Desktops.
- [ ] **Hyper-V-Konsole** (`/pcb`) und **Load-Balancing-Info** für RDS-Farmen.
- [ ] **Entra-ID-Anmeldung** (`/sec:aad`).

## Wayland-Linux

- [ ] **GNOME 42+:** gnome-remote-desktop ist ein eingebauter RDP-Server. Er
  teilt entweder die laufende Sitzung oder bietet ab GNOME 46 „Remote Login“
  mit eigener Sitzung über GDM. Für Remote Login muss der Client die
  Server-Umleitung beherrschen, das kann FreeRDP 3. Gegen einen echten
  GNOME-Wayland-Server testen.
