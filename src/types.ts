import { t } from "./i18n";
export type DisplayMode = "window" | "workarea" | "fullscreen";
export type Protocol = "rdp" | "vnc" | "ssh" | "sftp" | "mosh";
export type ResizeBehavior = "dynamic" | "scale" | "fixed";
// FreeRDP nimmt nur diese Werte entgegen und bricht sonst mit einem Argumentfehler ab.
export type ColorDepth = "auto" | "32" | "24" | "16" | "15" | "8";

/** Ein als Laufwerk in die Sitzung gereichter Ordner. */
export type SharedFolder = {
  name: string;
  path: string;
};

export type RemoteProfile = {
  id: string;
  name: string;
  protocol: Protocol;
  host: string;
  username: string;
  domain: string;
  rdpTcpPort: number;
  rdpUdpPort: number;
  vncPort: number;
  sshPort: number;
  sshTerminal: string;
  x11Forwarding: boolean;
  x11Command: string;
  x11CloseXquartz: boolean;
  udpPreferred: boolean;
  clipboard: boolean;
  audio: boolean;
  displayMode: DisplayMode;
  width: number;
  height: number;
  resizeBehavior: ResizeBehavior;
  colorDepth: ColorDepth;
  sharedFolders: SharedFolder[];
  printer: boolean;
  smartcard: boolean;
  video: boolean;
  /** FreeRDPs Wiederverbindung nach kurzem Aussetzer (dort standardmässig aus). */
  autoReconnect: boolean;
  gatewayEnabled: boolean;
  gatewayHost: string;
  gatewayPort: number;
  gatewayUsername: string;
  gatewayDomain: string;
  certificateMode: "prompt" | "tofu" | "ignore";
  updatedAt: string;
};

/** Gängige Zielauflösungen; FreeRDP verlangt gerade Werte. */
export const resolutionPresets = [
  { label: "1280 × 800", width: 1280, height: 800 },
  { label: "1440 × 900", width: 1440, height: 900 },
  { label: "1600 × 1000", width: 1600, height: 1000 },
  { label: "1920 × 1080", width: 1920, height: 1080 },
  { label: "2560 × 1440", width: 2560, height: 1440 },
] as const;

export const emptyProfile = (): RemoteProfile => ({
  id: crypto.randomUUID(),
  // Vorbelegung folgt der Oberflaechensprache; ab dem Speichern ist es ein Datenwert.
  name: t("top.untitled"),
  protocol: "rdp",
  host: "",
  username: "",
  domain: "",
  rdpTcpPort: 3389,
  rdpUdpPort: 3389,
  vncPort: 5900,
  sshPort: 22,
  sshTerminal: "xterm-256color",
  x11Forwarding: false,
  x11Command: "",
  x11CloseXquartz: false,
  udpPreferred: true,
  clipboard: true,
  audio: false,
  displayMode: "window",
  width: 1600,
  height: 1000,
  resizeBehavior: "dynamic",
  colorDepth: "auto",
  sharedFolders: [],
  printer: false,
  smartcard: false,
  video: false,
  autoReconnect: true,
  gatewayEnabled: false,
  gatewayHost: "",
  gatewayPort: 443,
  gatewayUsername: "",
  gatewayDomain: "",
  certificateMode: "tofu",
  updatedAt: new Date().toISOString(),
});
