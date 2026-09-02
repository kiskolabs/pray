import type { ManagedSpanRecord } from "../lockfile/types.js";

type Segment =
  | { kind: "text"; text: string }
  | { kind: "managed"; id: string; body: string };

export function patchRenderedContent(existing: string, fresh: string): string {
  const existingSegments = splitSegments(existing);
  const freshSegments = splitSegments(fresh);
  const freshManaged = new Map(
    freshSegments
      .filter((segment) => segment.kind === "managed")
      .map((segment) => [segment.id, segment.body]),
  );
  const overlaps = existingSegments.some(
    (segment) => segment.kind === "managed" && freshManaged.has(segment.id),
  );
  if (!overlaps) return fresh;

  const used = new Set<string>();
  let output = "";
  for (const segment of existingSegments) {
    if (segment.kind === "text") {
      output += segment.text;
      continue;
    }
    const body = freshManaged.get(segment.id) ?? segment.body;
    used.add(segment.id);
    output += managedSegment(segment.id, body);
  }
  for (const segment of freshSegments) {
    if (segment.kind === "managed" && !used.has(segment.id)) {
      output += managedSegment(segment.id, segment.body);
    }
  }
  return output.endsWith("\n") ? output : `${output}\n`;
}

export function relocateManagedSpans(
  content: string,
  spans: ManagedSpanRecord[],
): ManagedSpanRecord[] {
  const positions = markerPositions(linesOf(content));
  return spans.map((span) => {
    const position = positions.get(span.id);
    return position
      ? { ...span, open_line: position[0], close_line: position[1] }
      : span;
  });
}

function splitSegments(content: string): Segment[] {
  const lines = linesOf(content);
  const segments: Segment[] = [];
  let text = "";
  for (let index = 0; index < lines.length; index += 1) {
    const id = markerId(lines[index] ?? "");
    const close = id ? findClosingMarker(lines, index + 1, id) : undefined;
    if (id && close !== undefined) {
      if (text) segments.push({ kind: "text", text });
      text = "";
      const bodyLines = lines.slice(index + 1, close);
      const body = bodyLines.length > 0 ? `${bodyLines.join("\n")}\n` : "";
      segments.push({ kind: "managed", id, body });
      index = close;
    } else {
      text += `${lines[index] ?? ""}\n`;
    }
  }
  if (text) segments.push({ kind: "text", text });
  return segments;
}

function linesOf(content: string): string[] {
  if (!content) return [];
  const lines = content.split(/\r?\n/);
  if (content.endsWith("\n")) lines.pop();
  return lines;
}

function findClosingMarker(
  lines: string[],
  start: number,
  id: string,
): number | undefined {
  for (let index = start; index < lines.length; index += 1) {
    if (markerId(lines[index] ?? "") === id) return index;
  }
  return undefined;
}

function markerPositions(lines: string[]): Map<string, [number, number]> {
  const positions = new Map<string, [number, number]>();
  let active: [string, number] | undefined;
  lines.forEach((line, index) => {
    const id = markerId(line);
    if (!id || !/^[a-z0-9]+$/.test(id)) return;
    if (!active) active = [id, index + 1];
    else if (active[0] === id) {
      positions.set(id, [active[1], index + 1]);
      active = undefined;
    }
  });
  return positions;
}

function markerId(line: string): string | undefined {
  const match = /^<!-- pray:(.+) -->$/.exec(line.trim());
  const id = match?.[1];
  return id && id !== "0 ignore-comments" ? id : undefined;
}

function managedSegment(id: string, body: string): string {
  const content = body ? `${body.replace(/\n+$/, "")}\n` : "";
  return `<!-- pray:${id} -->\n${content}<!-- pray:${id} -->\n`;
}
