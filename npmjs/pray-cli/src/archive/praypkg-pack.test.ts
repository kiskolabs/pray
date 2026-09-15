import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { parsePackageSpec } from "../package-spec/index.js";
import type { ResolvedPackage } from "../resolve/types.js";
import { buildPackageArchiveBytes } from "./praypkg.js";

function packageEntry(root: string, specText: string): ResolvedPackage {
  const spec = parsePackageSpec(specText);
  return {
    declaration: {
      name: spec.name,
      constraint: spec.version,
      exports: [],
      targets: [],
      features: [],
      groups: [],
      optional: false,
    },
    root,
    spec,
    treeHash: "sha256:tree",
    artifactHash: "sha256:tree",
    artifact: `path:${root}`,
    selectedExports: [],
    sourceChecksum: "sha256:tree",
    exportBodies: new Map(),
    skillFiles: new Map(),
  };
}

describe("praypkg packing", () => {
  it("packs when spec.files lists the package spec", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-pack-listed-"));
    try {
      writeFileSync(
        join(root, "demo.prayspec"),
        `Package::Specification.new do |spec|
  spec.name = "demo"
  spec.version = "1.0.0"
  spec.files = ["demo.prayspec", "rules.md"]
end
`,
      );
      writeFileSync(join(root, "rules.md"), "# demo\n");
      const bytes = buildPackageArchiveBytes(
        packageEntry(
          root,
          `Package::Specification.new do |spec|
  spec.name = "demo"
  spec.version = "1.0.0"
  spec.files = ["demo.prayspec", "rules.md"]
end
`,
        ),
      );
      assert.ok(bytes.byteLength > 0);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects duplicate spec.files paths", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-pack-duplicate-"));
    try {
      mkdirSync(root, { recursive: true });
      writeFileSync(
        join(root, "demo.prayspec"),
        `Package::Specification.new do |spec|
  spec.name = "demo"
  spec.version = "1.0.0"
  spec.files = ["rules.md", "rules.md"]
end
`,
      );
      writeFileSync(join(root, "rules.md"), "# demo\n");
      assert.throws(
        () =>
          buildPackageArchiveBytes(
            packageEntry(
              root,
              `Package::Specification.new do |spec|
  spec.name = "demo"
  spec.version = "1.0.0"
  spec.files = ["rules.md", "rules.md"]
end
`,
            ),
          ),
        (error: unknown) =>
          error instanceof PrayError &&
          error.message.includes("duplicate package archive path"),
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects aliased spec.files paths", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-pack-alias-"));
    try {
      writeFileSync(
        join(root, "demo.prayspec"),
        `Package::Specification.new do |spec|
  spec.name = "demo"
  spec.version = "1.0.0"
  spec.files = ["./rules.md", "rules.md"]
end
`,
      );
      writeFileSync(join(root, "rules.md"), "# demo\n");
      assert.throws(
        () =>
          buildPackageArchiveBytes(
            packageEntry(
              root,
              `Package::Specification.new do |spec|
  spec.name = "demo"
  spec.version = "1.0.0"
  spec.files = ["./rules.md", "rules.md"]
end
`,
            ),
          ),
        (error: unknown) =>
          error instanceof PrayError &&
          error.message.includes("duplicate package archive path"),
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
