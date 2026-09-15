import assert from "node:assert/strict";
import { cpSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { buildLockfile } from "./lockfile/index.js";
import { defaultResolveOptions, resolveProject } from "./resolve/project.js";

const here = dirname(fileURLToPath(import.meta.url));
const fixturesRoot = join(here, "../../../fixtures");

describe("RFC 0100 distribution resolver fixtures", () => {
  it("resolves the git-distribution fixture into a lock slice", async () => {
    await assertCopiedLockSlice("git-distribution");
  });

  it("resolves the registry-distribution fixture into a lock slice", async () => {
    await assertCopiedLockSlice("registry-distribution");
  });

  it("resolves the tarball-package fixture into a lock slice", async () => {
    await assertCopiedLockSlice("tarball-package", true);
  });
});

async function assertCopiedLockSlice(
  pack: string,
  offline = false,
): Promise<void> {
  const source = join(fixturesRoot, "resolver", pack);
  const expected = JSON.parse(
    readFileSync(join(source, "expected.json"), "utf8"),
  );
  const copied = mkdtempSync(join(tmpdir(), `pray-${pack}-`));
  cpSync(source, copied, { recursive: true });
  const project = await resolveProject(join(copied, "Prayfile"), {
    ...defaultResolveOptions(),
    offline,
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
  const packages = lockfile.package
    .map((entry) => ({
      name: entry.name,
      version: entry.version,
      tree_hash: entry.tree_hash,
      exports: entry.exports,
    }))
    .sort((left, right) => left.name.localeCompare(right.name));
  assert.deepEqual(packages, expected.packages);
}
