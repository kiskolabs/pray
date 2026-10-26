import type { ManagedSpanRecord } from "../lockfile/types.js";
import type { ResolvedLocalFile } from "../resolve/types.js";

export interface PositionDriftSummary {
  targetPath: string;
  markerCount: number;
  uniformDelta: number | undefined;
  firstId: string;
  lockOpen: number;
  lockClose: number;
  fileOpen: number;
  fileClose: number;
  cause: string | undefined;
}

interface DriftedMarker {
  id: string;
  lockOpen: number;
  lockClose: number;
  fileOpen: number;
  fileClose: number;
}

export function summarizePositionDrift(
  targetPath: string,
  spans: ManagedSpanRecord[],
  markers: Map<
    string,
    { openLine: number; closeLine: number; checksum: string }
  >,
  onDiskLines: string[],
  freshLines: string[] | undefined,
  localFiles: ResolvedLocalFile[],
): PositionDriftSummary | undefined {
  const drifted: DriftedMarker[] = [];
  for (const span of spans) {
    const marker = markers.get(span.id);
    if (!marker) {
      continue;
    }
    if (marker.checksum !== span.ideal_checksum) {
      continue;
    }
    if (
      marker.openLine === span.open_line &&
      marker.closeLine === span.close_line
    ) {
      continue;
    }
    drifted.push({
      id: span.id,
      lockOpen: span.open_line,
      lockClose: span.close_line,
      fileOpen: marker.openLine,
      fileClose: marker.closeLine,
    });
  }
  if (drifted.length === 0) {
    return undefined;
  }
  drifted.sort((left, right) => left.fileOpen - right.fileOpen);
  const first = drifted[0]!;
  const deltas = drifted.map((marker) => marker.fileOpen - marker.lockOpen);
  const uniformDelta = deltas.every((delta) => delta === deltas[0])
    ? deltas[0]
    : undefined;
  const cause = freshLines
    ? unmarkedDriftCause(targetPath, onDiskLines, freshLines, localFiles)
    : undefined;
  return {
    targetPath,
    markerCount: drifted.length,
    uniformDelta,
    firstId: first.id,
    lockOpen: first.lockOpen,
    lockClose: first.lockClose,
    fileOpen: first.fileOpen,
    fileClose: first.fileClose,
    cause,
  };
}

export function formatPositionDriftMessage(
  summary: PositionDriftSummary,
): string {
  const shift =
    summary.uniformDelta !== undefined && summary.uniformDelta !== 0
      ? ` (${summary.uniformDelta > 0 ? "+" : ""}${summary.uniformDelta} lines)`
      : "";
  const markerWord = summary.markerCount === 1 ? "marker" : "markers";
  const parts = [
    `\`${summary.targetPath}\` position drift${shift} across ${summary.markerCount} ${markerWord}`,
    `first marker \`${summary.firstId}\` lock ${summary.lockOpen}:${summary.lockClose}, file ${summary.fileOpen}:${summary.fileClose}`,
  ];
  if (summary.cause) {
    parts.push(`cause: ${summary.cause}`);
  }
  parts.push(
    "Align unmarked text with compose sources, or run `pray install` to refresh lock positions.",
  );
  return parts.join("; ");
}

function unmarkedDriftCause(
  targetPath: string,
  onDiskLines: string[],
  freshLines: string[],
  localFiles: ResolvedLocalFile[],
): string | undefined {
  const diskPreamble = preambleLines(onDiskLines);
  const freshPreamble = preambleLines(freshLines);
  const diff = firstLineDiff(diskPreamble, freshPreamble);
  if (!diff) {
    return undefined;
  }
  const [index, diskLine, freshLine] = diff;
  const targetLine = index + 1;
  const freshLocal = locateLineInLocals(localFiles, freshLine);
  if (freshLocal) {
    return `\`${targetPath}:${targetLine}\` unmarked text differs from \`${freshLocal.path}:${freshLocal.line}\``;
  }
  const diskLocal = locateLineInLocals(localFiles, diskLine);
  if (diskLocal) {
    return `\`${targetPath}:${targetLine}\` unmarked text differs from \`${diskLocal.path}:${diskLocal.line}\``;
  }
  return `\`${targetPath}:${targetLine}\` unmarked text differs from fresh composition`;
}

function preambleLines(lines: string[]): string[] {
  const preamble: string[] = [];
  for (const line of lines) {
    if (isManagedMarker(line)) {
      break;
    }
    preamble.push(line);
  }
  return preamble;
}

function isManagedMarker(line: string): boolean {
  const trimmed = line.trim();
  if (!trimmed.startsWith("<!-- pray:") || !trimmed.endsWith(" -->")) {
    return false;
  }
  const id = trimmed.slice("<!-- pray:".length, -" -->".length);
  return id !== "0 ignore-comments" && /^[a-z0-9]+$/.test(id);
}

function firstLineDiff(
  left: string[],
  right: string[],
): [number, string, string] | undefined {
  const shared = Math.min(left.length, right.length);
  for (let index = 0; index < shared; index += 1) {
    if (left[index] !== right[index]) {
      return [index, left[index]!, right[index]!];
    }
  }
  if (left.length === right.length) {
    return undefined;
  }
  const index = shared;
  return [index, left[index] ?? "", right[index] ?? ""];
}

function locateLineInLocals(
  localFiles: ResolvedLocalFile[],
  line: string,
): { path: string; line: number } | undefined {
  if (line.length === 0) {
    return undefined;
  }
  for (const local of localFiles) {
    const lines = local.content.split("\n");
    for (let index = 0; index < lines.length; index += 1) {
      if (lines[index] === line) {
        return { path: local.manifestPath, line: index + 1 };
      }
    }
  }
  return undefined;
}
