import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";

const FORBIDDEN = new Set([
  "node:fs",
  "node:fs/promises",
  "node:child_process",
  "node:http",
  "node:https",
  "node:net",
  "node:crypto",
  "node:os",
  "node:stream",
]);

const IMPORT_PATTERN =
  /(?:from|import)\s*["']([^"']+)["']|import\s*\(\s*["']([^"']+)["']\s*\)/g;

describe("parse entry module graph", () => {
  it("does not import Node FS, process spawn, HTTP, or crypto builtins", () => {
    const sourceRoot = resolve(
      dirname(fileURLToPath(import.meta.url)),
      "../src",
    );
    const entry = join(sourceRoot, "parse.ts");
    const visited = new Set<string>();
    const queue = [entry];
    const forbiddenHits: string[] = [];

    while (queue.length > 0) {
      const file = queue.shift();
      if (file === undefined || visited.has(file)) {
        continue;
      }
      visited.add(file);
      const text = readFileSync(file, "utf8");
      for (const match of text.matchAll(IMPORT_PATTERN)) {
        const specifier = match[1] ?? match[2];
        if (specifier === undefined) {
          continue;
        }
        if (FORBIDDEN.has(specifier)) {
          forbiddenHits.push(`${file} -> ${specifier}`);
          continue;
        }
        if (!specifier.startsWith(".")) {
          continue;
        }
        const resolved = resolveSpecifier(file, specifier);
        if (resolved !== undefined) {
          queue.push(resolved);
        }
      }
    }

    assert.equal(
      forbiddenHits.join("\n"),
      "",
      `parse graph includes forbidden Node builtins:\n${forbiddenHits.join("\n")}`,
    );
    assert.ok(visited.size > 5, "expected a non-trivial parse module graph");
  });
});

function resolveSpecifier(
  fromFile: string,
  specifier: string,
): string | undefined {
  const base = resolve(dirname(fromFile), specifier);
  const candidates = [
    base,
    `${base}.ts`,
    join(base, "index.ts"),
    base.replace(/\.js$/, ".ts"),
  ];
  for (const candidate of candidates) {
    try {
      readFileSync(candidate);
      return candidate;
    } catch {
      // try next
    }
  }
  return undefined;
}
