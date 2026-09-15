import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { PrayError } from "./errors.js";
import { buildLockfile } from "./lockfile/index.js";
import { defaultResolveOptions, resolveProject } from "./resolve/project.js";

const here = dirname(fileURLToPath(import.meta.url));
const fixturesRoot = join(here, "../../../fixtures");

describe("RFC 0100 resolver fixtures", () => {
  it("resolves the path-package fixture into a lock slice", async () => {
    const dir = join(fixturesRoot, "resolver/path-package");
    const expected = JSON.parse(
      readFileSync(join(dir, "expected.json"), "utf8"),
    );
    const project = await resolveProject(join(dir, "Prayfile"), {
      ...defaultResolveOptions(),
      offline: true,
    });
    const lockfile = buildLockfile({
      manifestHash: project.manifestHash,
      environment: project.environment,
      projectRoot: project.projectRoot,
      manifestSources: project.manifest.sources,
      manifestTargets: project.manifest.targets,
      rendered: [],
      packages: project.packages,
      sourceRevisions: project.sourceRevisions,
      sourceHostKeys: project.sourceHostKeys,
    });
    const packages = lockfile.package.map((entry) => ({
      name: entry.name,
      version: entry.version,
      path: entry.path,
      tree_hash: entry.tree_hash,
      artifact: entry.artifact,
      exports: entry.exports,
    }));
    assert.deepEqual(packages, expected.packages);
  });

  it("rejects the constraint-mismatch resolver fixture", async () => {
    const dir = join(fixturesRoot, "resolver/constraint-mismatch");
    await assert.rejects(
      () =>
        resolveProject(join(dir, "Prayfile"), {
          ...defaultResolveOptions(),
          offline: true,
        }),
      PrayError,
    );
  });
});
