import { createHash } from "node:crypto";

export function sha256Hex(bytes: Buffer | string): string {
  return createHash("sha256").update(bytes).digest("hex");
}

export function sha256Prefixed(bytes: Buffer | string): string {
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
  const hasher = createHash("sha256");
  for (let index = 0; index < trimmed.length; index += 1) {
    if (index > 0) {
      hasher.update("\n");
    }
    const line = trimmed[index];
    if (line === undefined) {
      continue;
    }
    hasher.update(
      line.includes("\r") ? normalizeLineEndings(line) : line,
    );
  }
  return prefixedHexDigest(hasher.digest());
}

function prefixedHexDigest(digest: Buffer): string {
  return `sha256:${digest.toString("hex")}`;
}

function trimTrailingEmptyLines(lines: string[]): string[] {
  let end = lines.length;
  while (end > 0 && lines[end - 1] === "") {
    end -= 1;
  }
  return lines.slice(0, end);
}
