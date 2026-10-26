import { sha256HexBytes } from "./sha256.js";

export function sha256Hex(bytes: Uint8Array | string): string {
  return sha256HexBytes(bytes);
}

export function sha256Prefixed(bytes: Uint8Array | string): string {
  return `sha256:${sha256Hex(bytes)}`;
}

export function markerId(seed: string): string {
  return sha256Hex(seed).slice(0, 8);
}

export function normalizeLineEndings(text: string): string {
  return text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

export function checksumManagedSpanContent(body: string): string {
  const normalized = normalizeLineEndings(body).replace(/\n+$/, "");
  return sha256Prefixed(normalized);
}

export function checksumManagedBodyLineRefs(bodyLines: string[]): string {
  const trimmed = trimTrailingEmptyLines(bodyLines);
  const normalized = trimmed.map((line) =>
    line.includes("\r") ? normalizeLineEndings(line) : line,
  );
  return sha256Prefixed(normalized.join("\n"));
}

function trimTrailingEmptyLines(lines: string[]): string[] {
  let end = lines.length;
  while (end > 0 && lines[end - 1] === "") {
    end -= 1;
  }
  return lines.slice(0, end);
}
