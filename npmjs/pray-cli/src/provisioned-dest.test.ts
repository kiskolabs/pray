import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { provisionedChange } from "./cli/commands/plan.js";
import { materializeProject } from "./cli/materialize.js";
import { PrayError } from "./errors.js";
import { sha256Prefixed } from "./hashing.js";
import { buildLockfile } from "./lockfile/index.js";
import { defaultLockfilePath } from "./lockfile/paths.js";
import type { Lockfile } from "./lockfile/types.js";
import {
  validateDestinationPath,
  validateProjectRelativePath,
} from "./manifest/validate.js";
import {
  materializeProvisionedExports,
  provisionedLockRecords,
} from "./render/dest.js";
import { writeRenderedTargets } from "./render/project.js";
import { plannedProvisionedFiles } from "./render/provisioned.js";
import { resolveProject } from "./resolve/project.js";

function writeFilePackage(root: string, body: string): void {
  const packageRoot = join(root, "packages/shell");
  mkdirSync(join(packageRoot, "exports"), { recursive: true });
  writeFileSync(
    join(packageRoot, "shell.prayspec"),
    `
Package::Specification.new do |spec|
  spec.name = "sample/shell"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/zshrc"]
  spec.exports = {
    "zshrc" => { type: "file", path: "exports/zshrc" }
  }
end
`,
  );
  writeFileSync(join(packageRoot, "exports/zshrc"), body);
  writeFileSync(
    join(root, "Prayfile"),
    `
prayfile "1"
pray "sample/shell", "~> 1.0", path: "packages/shell", file: ".zshrc"
`,
  );
}

async function resolveRoot(root: string) {
  return resolveProject(join(root, "Prayfile"));
}

function lockfileWithProvisioned(
  project: Awaited<ReturnType<typeof resolveRoot>>,
): Lockfile {
  const lockfile = buildLockfile({
    manifestHash: project.manifestHash,
    projectRoot: project.projectRoot,
    manifestSources: project.manifest.sources,
    manifestTargets: project.manifest.targets,
    rendered: [],
    packages: project.packages,
  });
  return {
    ...lockfile,
    provisioned: provisionedLockRecords(project),
  };
}

describe("provisioned destination safety", () => {
  it("rejects a leading tilde only in a destination path", () => {
    assert.equal(
      validateProjectRelativePath("~fixtures/shell"),
      "~fixtures/shell",
    );
    assert.throws(
      () => validateDestinationPath("~/.zshrc"),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("repository-relative"),
    );
  });

  it("writes an exclusive file when the dest is missing", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-missing-"));
    writeFilePackage(root, "alias ll=ls\n");
    const project = await resolveRoot(root);
    writeRenderedTargets(project, []);
    assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "alias ll=ls\n");
  });

  it("adopts a dest whose bytes already match expected", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-adopt-"));
    writeFilePackage(root, "alias ll=ls\n");
    writeFileSync(join(root, ".zshrc"), "alias ll=ls\n");
    const project = await resolveRoot(root);
    writeRenderedTargets(project, []);
    assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "alias ll=ls\n");
  });

  it("refuses to clobber unmanaged dest bytes", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-clobber-"));
    writeFilePackage(root, "alias ll=ls\n");
    writeFileSync(join(root, ".zshrc"), "keep me\n");
    const project = await resolveRoot(root);
    assert.throws(
      () => writeRenderedTargets(project, []),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes(".zshrc"),
    );
    assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "keep me\n");
  });

  it("refuses a symlink destination", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-symlink-"));
    writeFilePackage(root, "alias ll=ls\n");
    const target = join(root, "real-zshrc");
    writeFileSync(target, "keep link target\n");
    symlinkSync(target, join(root, ".zshrc"));
    const project = await resolveRoot(root);
    assert.throws(
      () => writeRenderedTargets(project, []),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes("symbolic link"),
    );
    assert.equal(readFileSync(target, "utf8"), "keep link target\n");
  });

  it("refuses a symlinked parent directory", async () => {
    const base = mkdtempSync(join(tmpdir(), "pray-provisioned-parent-link-"));
    const root = join(base, "project");
    const outside = join(base, "outside");
    mkdirSync(root);
    mkdirSync(outside);
    writeFilePackage(root, "alias ll=ls\n");
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
pray "sample/shell", "~> 1.0", path: "packages/shell", file: "linked/zshrc"
`,
    );
    symlinkSync(outside, join(root, "linked"), "dir");
    const project = await resolveRoot(root);

    assert.throws(
      () => writeRenderedTargets(project, []),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes("symbolic link"),
    );
    assert.equal(existsSync(join(outside, "zshrc")), false);
  });

  it("updates when the previous lock hash still matches", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-update-"));
    writeFilePackage(root, "alias ll=ls\n");
    const project = await resolveRoot(root);
    writeRenderedTargets(project, []);
    const lockfile = lockfileWithProvisioned(project);
    writeFileSync(join(root, "packages/shell/exports/zshrc"), "alias la=ls\n");
    const updated = await resolveRoot(root);
    writeRenderedTargets(updated, [], lockfile);
    assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "alias la=ls\n");
  });

  it("refuses a user-edited managed dest", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-edited-"));
    writeFilePackage(root, "alias ll=ls\n");
    const project = await resolveRoot(root);
    writeRenderedTargets(project, []);
    const lockfile = lockfileWithProvisioned(project);
    writeFileSync(join(root, ".zshrc"), "my aliases\n");
    writeFileSync(join(root, "packages/shell/exports/zshrc"), "alias la=ls\n");
    const updated = await resolveRoot(root);
    assert.throws(
      () => writeRenderedTargets(updated, [], lockfile),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes(".zshrc"),
    );
    assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "my aliases\n");
  });

  it("prunes a matching leaf and keeps an edited dest", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-prune-"));
    writeFilePackage(root, "alias ll=ls\n");
    let project = await resolveRoot(root);
    writeRenderedTargets(project, []);
    let lockfile = lockfileWithProvisioned(project);
    writeFileSync(join(root, "Prayfile"), 'prayfile "1"\n');
    let empty = await resolveRoot(root);
    writeRenderedTargets(empty, [], lockfile);
    assert.equal(existsSync(join(root, ".zshrc")), false);

    writeFilePackage(root, "alias ll=ls\n");
    project = await resolveRoot(root);
    writeRenderedTargets(project, []);
    lockfile = lockfileWithProvisioned(project);
    writeFileSync(join(root, ".zshrc"), "my aliases\n");
    writeFileSync(join(root, "Prayfile"), 'prayfile "1"\n');
    empty = await resolveRoot(root);
    writeRenderedTargets(empty, [], lockfile);
    assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "my aliases\n");
  });

  it("rejects a lock path outside the project before pruning", async () => {
    const base = mkdtempSync(join(tmpdir(), "pray-provisioned-lock-escape-"));
    const root = join(base, "project");
    mkdirSync(root);
    const outside = join(base, "outside.txt");
    writeFileSync(outside, "keep me\n");
    writeFileSync(join(root, "Prayfile"), 'prayfile "1"\n');
    const project = await resolveRoot(root);
    const lockfile = {
      provisioned: [
        {
          path: "../outside.txt",
          content_hash: sha256Prefixed("keep me\n"),
          package: "sample/shell",
          export: "zshrc",
        },
      ],
    } as Lockfile;

    assert.throws(
      () => materializeProvisionedExports(project, lockfile),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes("escapes"),
    );
    assert.equal(readFileSync(outside, "utf8"), "keep me\n");
  });

  it("reports a provisioned refusal instead of an update", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-plan-refusal-"));
    writeFilePackage(root, "package aliases\n");
    writeFileSync(join(root, ".zshrc"), "operator aliases\n");
    const project = await resolveRoot(root);
    const [file] = plannedProvisionedFiles(project);
    assert.ok(file);

    assert.throws(
      () => provisionedChange(project, file, undefined),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("refusing to overwrite `.zshrc`"),
    );
  });

  it("keeps the previous lock when destination materialization fails", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-retry-"));
    writeFilePackage(root, "old aliases\n");
    const previousWorkingDirectory = process.cwd();
    process.chdir(root);
    try {
      await materializeProject({ manifestPath: join(root, "Prayfile") });
      const lockfilePath = defaultLockfilePath(root);
      const previousLock = readFileSync(lockfilePath, "utf8");
      writeFileSync(
        join(root, "packages/shell/exports/zshrc"),
        "new aliases\n",
      );
      rmSync(join(root, ".zshrc"));
      mkdirSync(join(root, ".zshrc"));

      await assert.rejects(() =>
        materializeProject({ manifestPath: join(root, "Prayfile") }),
      );
      assert.equal(readFileSync(lockfilePath, "utf8"), previousLock);

      rmSync(join(root, ".zshrc"), { recursive: true });
      writeFileSync(join(root, ".zshrc"), "old aliases\n");
      await materializeProject({ manifestPath: join(root, "Prayfile") });
      assert.equal(readFileSync(join(root, ".zshrc"), "utf8"), "new aliases\n");
    } finally {
      process.chdir(previousWorkingDirectory);
    }
  });

  it("records path, hash, package, and export", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-provisioned-lock-"));
    writeFilePackage(root, "alias ll=ls\n");
    const project = await resolveRoot(root);
    const planned = plannedProvisionedFiles(project);
    assert.equal(planned.length, 1);
    const records = provisionedLockRecords(project);
    assert.deepEqual(records, [
      {
        path: ".zshrc",
        content_hash: sha256Prefixed("alias ll=ls\n"),
        package: "sample/shell",
        export: "zshrc",
      },
    ]);
  });
});
