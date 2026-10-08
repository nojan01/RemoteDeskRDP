// Erzeugt das Updater-Manifest `latest.json` für ein GitHub-Release.
//
// Aufruf:
//   npm run make-updater-manifest -- <version> <plattformen> <archiv> [ausgabe]
//
// <plattformen> ist eine Tauri-Plattformkennung oder eine Komma-Liste, etwa
// "darwin-aarch64" oder bei einer Universal-App "darwin-aarch64,darwin-x86_64".
// Neben <archiv> muss die Signatur <archiv>.sig liegen (`tauri signer sign`).
// Die Versionshinweise stammen aus dem passenden Abschnitt in CHANGELOG.md.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repository = "nojan01/RemoteDeskRDP";
const [version, platformList, artifact, output = "latest.json"] = process.argv.slice(2);

const fail = (...lines) => {
  for (const line of lines) console.error(line);
  process.exit(1);
};

if (!version || !platformList || !artifact) {
  fail(
    "Aufruf: npm run make-updater-manifest -- <version> <plattformen> <archiv> [ausgabe]",
    'Beispiel: npm run make-updater-manifest -- 0.7.0 darwin-aarch64 "…/RemoteDeskRDP.app.tar.gz"',
  );
}

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
  fail(`Keine gültige semantische Version: ${version}`);
}

const platforms = platformList.split(",").map((item) => item.trim()).filter(Boolean);
const unknown = platforms.filter((item) => !/^(darwin|linux|windows)-(aarch64|x86_64|i686|armv7)$/.test(item));
if (platforms.length === 0 || unknown.length > 0) {
  fail(`Unbekannte Plattformkennung: ${unknown.join(", ") || platformList}`);
}

const artifactPath = resolve(artifact);
if (!existsSync(artifactPath)) fail(`Updater-Archiv nicht gefunden: ${artifactPath}`);

const signaturePath = `${artifactPath}.sig`;
if (!existsSync(signaturePath)) {
  fail(
    `Signaturdatei nicht gefunden: ${signaturePath}`,
    "Signieren mit: npx tauri signer sign -f ~/.tauri/remotedesk-updater.key -p \"\" <archiv>",
  );
}

const signature = readFileSync(signaturePath, "utf8").trim();
if (!signature) fail(`Signatur ist leer: ${signaturePath}`);

/** Abschnitt "## <version> – Datum" aus dem Changelog, ohne Überschrift. */
function changelogNotes() {
  const changelog = resolve(dirname(fileURLToPath(import.meta.url)), "..", "CHANGELOG.md");
  if (!existsSync(changelog)) return "";
  const lines = readFileSync(changelog, "utf8").split("\n");
  const escaped = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const start = lines.findIndex((line) => new RegExp(`^##\\s+\\[?${escaped}\\]?(\\s|$)`).test(line));
  if (start < 0) return "";
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => /^##\s/.test(line));
  return (end < 0 ? rest : rest.slice(0, end)).join("\n").trim();
}

const assetName = basename(artifactPath);
const url = `https://github.com/${repository}/releases/download/v${version}/${encodeURIComponent(assetName)}`;
const manifest = {
  version,
  notes: changelogNotes() || `RemoteDeskRDP ${version}`,
  pub_date: new Date().toISOString(),
  platforms: Object.fromEntries(platforms.map((platform) => [platform, { signature, url }])),
};

const outputPath = resolve(output);
writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Updater-Manifest erstellt: ${outputPath}`);
