export type DeskletPosition = { x: number; y: number };
export type WorkArea = DeskletPosition & { width: number; height: number };

export function parseDeskletPosition(value: unknown): DeskletPosition | null {
  if (!value || typeof value !== "object") return null;
  const position = value as DeskletPosition;
  return Number.isSafeInteger(position.x) && Number.isSafeInteger(position.y)
    ? { x: position.x, y: position.y } : null;
}

/** Physische Koordinaten wie bei Tauri; negative Werte gehören zu Monitoren
 * links/oberhalb des Hauptbildschirms und sind ausdrücklich erlaubt. */
export function visibleDeskletPosition(position: DeskletPosition, size: { width: number; height: number }, areas: WorkArea[]): DeskletPosition {
  if (!areas.length) return position;
  const candidates = areas.map(area => {
    const x = Math.max(area.x, Math.min(position.x, area.x + Math.max(0, area.width - size.width)));
    const y = Math.max(area.y, Math.min(position.y, area.y + Math.max(0, area.height - size.height)));
    return { x, y, distance: (x - position.x) ** 2 + (y - position.y) ** 2 };
  });
  const nearest = candidates.reduce((best, candidate) => candidate.distance < best.distance ? candidate : best);
  return { x: nearest.x, y: nearest.y };
}
