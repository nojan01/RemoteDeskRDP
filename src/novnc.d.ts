/** Minimale Typen für die genutzte noVNC-Oberfläche (das Paket liefert keine). */
declare module "@novnc/novnc" {
  export interface RFBCredentials { username?: string; password?: string; target?: string }
  export default class RFB {
    constructor(target: HTMLElement, url: string, options?: { credentials?: RFBCredentials; shared?: boolean; wsProtocols?: string[] });
    scaleViewport: boolean;
    resizeSession: boolean;
    viewOnly: boolean;
    disconnect(): void;
    sendCredentials(credentials: RFBCredentials): void;
    addEventListener(type: string, listener: (event: CustomEvent) => void): void;
    removeEventListener(type: string, listener: (event: CustomEvent) => void): void;
  }
}
