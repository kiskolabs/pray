import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { parseManifest } from "./manifest/index.js";

const here = dirname(fileURLToPath(import.meta.url));
const corpusRoot = join(here, "../../../testdata/shared/manifest");

type ExpectedEntry = {
  kind: string;
  name?: string;
  path?: string;
};

type ExpectedCorpus = {
  targets: Array<{
    name: string;
    mode: string;
    scoped: boolean;
    outputs?: string[];
    skills?: string[];
    entries: ExpectedEntry[];
  }>;
  packages: Array<{
    name: string;
    bound: boolean;
    roles: string[];
    file?: string;
    path?: string;
  }>;
  local: Array<{ path: string; bound: boolean }>;
};

describe("shared fixture corpus", () => {
  const cases = readdirSync(corpusRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();

  for (const caseName of cases) {
    it(`parses ${caseName} against expected.json`, () => {
      const dir = join(corpusRoot, caseName);
      const text = readFileSync(join(dir, "Prayfile"), "utf8");
      const expected = JSON.parse(
        readFileSync(join(dir, "expected.json"), "utf8"),
      ) as ExpectedCorpus;
      const manifest = parseManifest(text);

      assert.equal(manifest.targets.length, expected.targets.length);
      for (const [index, want] of expected.targets.entries()) {
        const target = manifest.targets[index]!;
        assert.equal(target.name, want.name);
        assert.equal(target.mode ?? "legacy", want.mode);
        assert.equal(Boolean(target.scoped), want.scoped);
        assert.deepEqual(target.outputs ?? [], want.outputs ?? []);
        assert.deepEqual(target.skills ?? [], want.skills ?? []);
        assert.deepEqual(target.entries ?? [], want.entries);
      }

      assert.equal(manifest.packages.length, expected.packages.length);
      for (const [index, want] of expected.packages.entries()) {
        const packageDecl = manifest.packages[index]!;
        assert.equal(packageDecl.name, want.name);
        assert.equal(Boolean(packageDecl.bound), want.bound);
        assert.deepEqual(packageDecl.roles ?? [], want.roles);
        assert.equal(packageDecl.file, want.file);
        assert.equal(packageDecl.path, want.path);
      }

      assert.equal(manifest.local.length, expected.local.length);
      for (const [index, want] of expected.local.entries()) {
        const local = manifest.local[index]!;
        assert.equal(local.path, want.path);
        assert.equal(local.bound, want.bound);
      }
    });
  }
});
