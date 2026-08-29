// Theme-Verwaltung: Auto (folgt macOS), Hell, Dunkel.
// Gleiche Bauart wie in DualBeam, damit beide Apps sich gleich anfühlen.
// Eigener Speicherschlüssel, damit die Einstellungen nicht kollidieren.

import { createSignal } from "solid-js";

export type ThemeMode = "auto" | "light" | "dark";
const STORAGE_KEY = "remotedesk:theme:v1";

const [mode, setMode] = createSignal<ThemeMode>("auto");

function systemPrefersLight(): boolean {
  return typeof window !== "undefined"
    && !!window.matchMedia
    && window.matchMedia("(prefers-color-scheme: light)").matches;
}

function resolve(m: ThemeMode): "light" | "dark" {
  if (m === "auto") return systemPrefersLight() ? "light" : "dark";
  return m;
}

function apply(m: ThemeMode) {
  setMode(m);
  document.documentElement.dataset.theme = resolve(m);
}

export function initTheme() {
  let saved: ThemeMode = "auto";
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value === "auto" || value === "light" || value === "dark") saved = value;
  } catch {
    // localStorage nicht verfügbar – Standard "auto" beibehalten.
  }
  apply(saved);

  if (window.matchMedia) {
    // Im Auto-Modus dem Systemwechsel folgen.
    window.matchMedia("(prefers-color-scheme: light)")
      .addEventListener("change", () => { if (mode() === "auto") apply("auto"); });
  }
}

export function getThemeMode(): ThemeMode { return mode(); }

export function setThemeMode(m: ThemeMode) {
  try { localStorage.setItem(STORAGE_KEY, m); } catch {
    // Persistenz fehlgeschlagen – nicht kritisch, das Thema greift trotzdem.
  }
  apply(m);
}

export function cycleThemeMode() {
  const order: ThemeMode[] = ["auto", "light", "dark"];
  setThemeMode(order[(order.indexOf(mode()) + 1) % order.length]);
}

export function themeIcon(m: ThemeMode): string {
  return m === "light" ? "☀︎" : m === "dark" ? "☾" : "◐";
}
