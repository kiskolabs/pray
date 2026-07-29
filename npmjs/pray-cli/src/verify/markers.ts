import { checksumManagedBodyLineRefs } from "../hashing.js";

export function markerPositions(
  lines: string[],
): Map<string, { openLine: number; closeLine: number; checksum: string }> {
  const markers = new Map<
    string,
    { openLine: number; closeLine: number; checksum: string }
  >();
  let active: { id: string; openLine: number; body: string[] } | undefined;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]!;
    const parsed = parseMarker(line);
    if (!parsed) {
      active?.body.push(line);
      continue;
    }
    if (parsed === "ignore") {
      continue;
    }
    if (!active) {
      active = { id: parsed, openLine: index + 1, body: [] };
      continue;
    }
    if (active.id === parsed) {
      markers.set(active.id, {
        openLine: active.openLine,
        closeLine: index + 1,
        checksum: checksumManagedBodyLineRefs(active.body),
      });
      active = undefined;
    }
  }

  return markers;
}

function parseMarker(line: string): string | "ignore" | undefined {
  const trimmed = line.trim();
  if (!trimmed.startsWith("<!-- pray:") || !trimmed.endsWith(" -->")) {
    return undefined;
  }
  const id = trimmed.slice("<!-- pray:".length, -" -->".length);
  if (id === "0 ignore-comments") {
    return "ignore";
  }
  if (/^[a-z0-9]+$/.test(id)) {
    return id;
  }
  return undefined;
}
