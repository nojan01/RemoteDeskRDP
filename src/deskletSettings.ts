export type VmTarget = { id: string; name: string; address: string; interval: number };
export type DeskletSettings = { vms: VmTarget[]; opacity: number };

export function newVmTarget(): VmTarget {
  return { id: crypto.randomUUID(), name: "", address: "", interval: 10 };
}

/** Übernimmt auch die bis zur Mehrfachüberwachung gespeicherte Einzel-VM. */
export function parseDeskletSettings(value: unknown): DeskletSettings {
  if (!value || typeof value !== "object") return { vms: [], opacity: 100 };
  const stored = value as Record<string, unknown>;
  const opacity = typeof stored.opacity === "number" && Number.isInteger(stored.opacity)
    && stored.opacity >= 0 && stored.opacity <= 100 ? stored.opacity : 100;
  const entries = Array.isArray(stored.vms) ? stored.vms : [stored];
  const ids = new Set<string>();
  const vms: VmTarget[] = [];
  for (const entry of entries) {
    if (!entry || typeof entry !== "object" || typeof entry.address !== "string"
      || !entry.address.trim() || !Number.isInteger(entry.interval)
      || entry.interval < 3 || entry.interval > 3600) continue;
    const id = typeof entry.id === "string" && entry.id && !ids.has(entry.id) ? entry.id : crypto.randomUUID();
    ids.add(id);
    vms.push({ id, name: typeof entry.name === "string" ? entry.name.slice(0, 80) : "",
      address: entry.address.trim(), interval: entry.interval });
  }
  return { vms, opacity };
}
