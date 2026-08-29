// Sprachverwaltung: Auto (folgt macOS), Deutsch, Englisch.
// Gleiche Bauart wie in DualBeam. Eigener Speicherschlüssel.

import { createSignal } from "solid-js";
import { de } from "./locale/de";
import { en } from "./locale/en";

export type LangMode = "auto" | "de" | "en";
export type ResolvedLang = "de" | "en";

const STORAGE_KEY = "remotedesk:lang:v1";
const dicts: Record<ResolvedLang, Record<string, string>> = { de, en };

const [mode, setMode] = createSignal<LangMode>("auto");
const [resolved, setResolved] = createSignal<ResolvedLang>("de");

function systemLang(): ResolvedLang {
  try {
    return (navigator.language || "en").toLowerCase().startsWith("de") ? "de" : "en";
  } catch {
    return "en";
  }
}

function apply(m: LangMode) {
  const r = m === "auto" ? systemLang() : m;
  setMode(m);
  setResolved(r);
  try { document.documentElement.lang = r; } catch {
    // DOM eventuell noch nicht bereit – unkritisch.
  }
}

export function initI18n() {
  let saved: LangMode = "auto";
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value === "auto" || value === "de" || value === "en") saved = value;
  } catch {
    // localStorage nicht verfügbar – Standard "auto" beibehalten.
  }
  apply(saved);

  try {
    window.addEventListener("languagechange", () => { if (mode() === "auto") apply("auto"); });
  } catch {
    // Kein Ereignis verfügbar – Systemwechsel bleibt dann unbeobachtet.
  }
}

export function getLangMode(): LangMode { return mode(); }

export function setLangMode(m: LangMode) {
  try { localStorage.setItem(STORAGE_KEY, m); } catch {
    // Persistenz fehlgeschlagen – nicht kritisch.
  }
  apply(m);
}

export function cycleLangMode() {
  const order: LangMode[] = ["auto", "de", "en"];
  setLangMode(order[(order.indexOf(mode()) + 1) % order.length]);
}

export function langIcon(m: LangMode): string {
  return m === "auto" ? "🌐" : m === "de" ? "DE" : "EN";
}

/** Übersetzt einen Schlüssel. Liest resolved() – dadurch verfolgt Solid die Sprache
 *  und alle Beschriftungen wechseln ohne Neuladen. */
export function t(key: string, params?: Record<string, string | number>): string {
  let s = dicts[resolved()][key] ?? en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
    }
  }
  return s;
}

/** Wandelt einen abgefangenen Fehlerwert in eine lesbare, übersetzte Meldung. */
export function errMsg(e: unknown): string {
  let raw: string;
  if (e instanceof Error) raw = e.message;
  else if (typeof e === "string") raw = e;
  else if (e == null) return String(e);
  else {
    try { raw = typeof e === "object" ? JSON.stringify(e) : String(e); }
    catch { return String(e); }
  }
  return translateErr(raw);
}

/** Übersetzt einen Fehlercode des Backends (`err.*`). Parameter hängen hinter
 *  dem Unit-Separator (\x1f) und ersetzen im Text {0}, {1} … Unbekannte oder
 *  freie Texte bleiben unverändert. */
export function translateErr(raw: string): string {
  const parts = raw.trim().split("\x1f");
  const code = parts[0];
  if (code.startsWith("err.")) {
    let msg = dicts[resolved()][code] ?? en[code];
    if (msg !== undefined) {
      for (let i = 1; i < parts.length; i++) {
        msg = msg.replace(new RegExp(`\\{${i - 1}\\}`, "g"), parts[i]);
      }
      return msg;
    }
  }
  return raw;
}
