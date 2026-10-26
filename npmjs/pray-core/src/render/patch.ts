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

  const existingIds = new Set(
    existingSegments
      .filter((segment) => segment.kind === "managed")
      .map((segment) => segment.id),
  );
  const used = new Set<string>();
  let output = "";
  let remaining = existingSegments;
  if ([...freshManaged.keys()].some((id) => !existingIds.has(id))) {
    const shared = freshSegments.find(
      (segment) => segment.kind === "managed" && existingIds.has(segment.id),
    );
    if (shared?.kind !== "managed") return fresh;
    for (const segment of freshSegments) {
      if (segment.kind === "managed" && segment.id === shared.id) break;
      if (segment.kind === "text") {
        output += segment.text;
        continue;
      }
      used.add(segment.id);
      output += managedSegment(segment.id, segment.body);
    }
    remaining = skipUntilManaged(existingSegments, shared.id);
  }
  for (let index = 0; index < remaining.length; index += 1) {
    const segment = remaining[index];
    if (!segment) continue;
    if (segment.kind === "text") {
      output += segment.text;
      continue;
    }
    const body = freshManaged.get(segment.id) ?? segment.body;
    used.add(segment.id);
    output += managedSegment(segment.id, body);
    const next = remaining[index + 1];
    if (next?.kind === "managed") {
      output += textBetween(freshSegments, segment.id, next.id);
    }
  }
  output += unusedFreshSuffix(freshSegments, used);
  return output.endsWith("\n") ? output : `${output}\n`;
}

function textBetween(segments: Segment[], left: string, right: string): string {
  let copying = false;
  let text = "";
  for (const segment of segments) {
    if (segment.kind === "managed") {
      if (segment.id === left) {
        copying = true;
        text = "";
      } else if (copying) {
        return segment.id === right ? text : "";
      }
    } else if (copying) {
      text += segment.text;
    }
  }
  return "";
}

function unusedFreshSuffix(
  freshSegments: Segment[],
  used: Set<string>,
): string {
  let pending = "";
  let seenUsed = false;
  let suffix = "";
  for (const segment of freshSegments) {
    if (segment.kind === "managed") {
      if (used.has(segment.id)) {
        seenUsed = true;
        pending = "";
        continue;
      }
      suffix += pending;
      pending = "";
      suffix += managedSegment(segment.id, segment.body);
      continue;
    }
    if (seenUsed) pending += segment.text;
  }
  return suffix;
}

function skipUntilManaged(segments: Segment[], id: string): Segment[] {
  const index = segments.findIndex(
    (segment) => segment.kind === "managed" && segment.id === id,
  );
  return index >= 0 ? segments.slice(index) : segments;
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
