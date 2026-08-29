#!/usr/bin/env node
// Hält die beiden Wörterbücher deckungsgleich. Läuft vor jedem Bau, damit ein
// vergessener Schlüssel nicht erst im laufenden Programm auffällt.
//
// Die Locale-Dateien sind TypeScript, tragen aber nur eine einzige
// Typangabe. Die wird entfernt, dann lässt sich die Datei als ESM laden –
// das erspart einen Testläufer als zusätzliche Abhängigkeit.

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

async function load(name) {
  const file = join(root, "src", "locale", `${name}.ts`);
  const source = await readFile(file, "utf8");
  const marker = `export const ${name}: Record<string, string> =`;
  if (!source.includes(marker)) {
    throw new Error(`${name}.ts hat eine unerwartete Form – "${marker}" fehlt.`);
  }
  const js = source.replace(marker, `export const ${name} =`);
  const module = await import(`data:text/javascript;base64,${Buffer.from(js).toString("base64")}`);
  return module[name];
}

const [de, en] = await Promise.all([load("de"), load("en")]);
const fehler = [];

const fehltInEn = Object.keys(de).filter((k) => !(k in en));
const fehltInDe = Object.keys(en).filter((k) => !(k in de));
if (fehltInEn.length) fehler.push(`Fehlt in en.ts: ${fehltInEn.join(", ")}`);
if (fehltInDe.length) fehler.push(`Fehlt in de.ts: ${fehltInDe.join(", ")}`);

const leer = [...Object.entries(de), ...Object.entries(en)]
  .filter(([, v]) => typeof v !== "string" || v.trim() === "")
  .map(([k]) => k);
if (leer.length) fehler.push(`Ohne Text: ${leer.join(", ")}`);

// Ein deutscher Text, der wörtlich auch englisch dasteht, ist fast immer eine
// vergessene Übersetzung. Ausgenommen sind Bezeichner und reine Kürzel.
const gleichErlaubt = new Set([
  "brand.subtitle", "transport.eyebrow", "gateway.eyebrow", "help.eyebrow",
  "unit.port", "unit.px", "help.dualbeam.h",
  "field.optional", "help.ports.h",
]);
const unuebersetzt = Object.keys(de)
  .filter((k) => k in en && !gleichErlaubt.has(k) && de[k] === en[k]);
if (unuebersetzt.length) fehler.push(`Nicht übersetzt: ${unuebersetzt.join(", ")}`);

// Platzhalter wie {0} oder {name} müssen beidseitig gleich sein, sonst bleibt
// beim Einsetzen eine Lücke stehen.
const platzhalter = (s) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort().join(",");
const abweichend = Object.keys(de)
  .filter((k) => k in en && platzhalter(de[k]) !== platzhalter(en[k]));
if (abweichend.length) fehler.push(`Platzhalter weichen ab: ${abweichend.join(", ")}`);

if (fehler.length) {
  console.error("Wörterbücher fehlerhaft:");
  for (const f of fehler) console.error(`  - ${f}`);
  process.exit(1);
}

console.log(`Wörterbücher in Ordnung (${Object.keys(de).length} Schlüssel, de + en).`);
