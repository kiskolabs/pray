import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { checkImportCycles } from "./check-import-cycles.mjs";

const npmjsRoot = resolve(fileURLToPath(new URL(".", import.meta.url)), "..");

describe("checkImportCycles", () => {
  it("detects a relative import cycle in a fixture tree", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-cycles-"));
    try {
      const src = join(root, "src");
      mkdirSync(src);
      writeFileSync(join(src, "a.ts"), 'import { b } from "./b.js";\nexport const a = b;\n');
      writeFileSync(join(src, "b.ts"), 'import { a } from "./a.js";\nexport const b = a;\n');
      const cycles = checkImportCycles(src, [join(src, "a.ts")]);
      assert.equal(cycles.length, 1);
      assert.ok(cycles[0].join(" ").includes("a.ts"));
      assert.ok(cycles[0].join(" ").includes("b.ts"));
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("matches madge: no cycles from pray-core entrypoints", () => {
    const src = join(npmjsRoot, "pray-core/src");
    const cycles = checkImportCycles(src, [
      join(src, "index.ts"),
      join(src, "parse.ts"),
    ]);
    assert.deepEqual(cycles, []);
  });

  it("matches madge: no cycles from pray-cli entrypoints", () => {
    const src = join(npmjsRoot, "pray-cli/src");
    const cycles = checkImportCycles(src, [
      join(src, "cli/main.ts"),
      join(src, "parse.ts"),
    ]);
    assert.deepEqual(cycles, []);
  });
});
