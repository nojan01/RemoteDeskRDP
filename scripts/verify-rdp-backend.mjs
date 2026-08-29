import { access, readdir } from "node:fs/promises";
import { constants } from "node:fs";
import { execFile } from "node:child_process";
import { join, resolve } from "node:path";
import { promisify } from "node:util";

const run = promisify(execFile);

const root = "src-tauri/resources/freerdp/MacFreeRDP.app/Contents";
// sdl-freerdp ist das bevorzugte Backend, MacFreeRDP der Rückfall. Ohne die
// .nib-Dateien startet der native Client ohne Menü und ohne Zertifikats- bzw.
// Kennwortdialog.
const required = [
  [`${root}/MacOS/sdl-freerdp`, constants.X_OK],
  [`${root}/MacOS/MacFreeRDP`, constants.X_OK],
  [`${root}/Info.plist`, constants.R_OK],
  [`${root}/Resources/MainMenu.nib`, constants.R_OK],
];

const missing = [];
for (const [path, mode] of required) {
  try {
    await access(resolve(path), mode);
  } catch {
    missing.push(path);
  }
}

if (missing.length > 0) {
  console.error(
    `The self-contained FreeRDP backend is incomplete (${missing.join(", ")}). ` +
      "Run `npm run build:rdp-backend` before packaging RemoteDeskRDP.",
  );
  process.exitCode = 1;
}

// A backend that still links against Homebrew or XQuartz works on the build
// machine and nowhere else. Everything has to resolve inside the bundle.
const bundled = ["MacOS", "Frameworks"];
const external = [];
for (const directory of bundled) {
  const base = resolve(root, directory);
  let entries = [];
  try {
    entries = await readdir(base, { withFileTypes: true, recursive: true });
  } catch {
    continue;
  }
  for (const entry of entries) {
    if (!entry.isFile()) continue;
    const file = join(entry.parentPath ?? base, entry.name);
    let stdout = "";
    try {
      ({ stdout } = await run("otool", ["-L", file]));
    } catch {
      continue; // Keine Mach-O-Datei.
    }
    for (const line of stdout.split("\n")) {
      if (!line.startsWith("\t")) continue; // Architektur-Kopfzeilen überspringen.
      const path = line.trim().split(" ")[0];
      if (/^(@rpath|@executable_path|@loader_path|\/usr\/lib\/|\/System\/)/.test(path)) continue;
      external.push(`${entry.name} -> ${path}`);
    }
  }
}

if (external.length > 0) {
  console.error(
    "The FreeRDP backend depends on libraries outside the bundle:\n  " +
      [...new Set(external)].join("\n  "),
  );
  process.exitCode = 1;
}
